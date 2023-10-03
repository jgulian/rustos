use core::fmt;
use core::fmt::Formatter;

use kernel_api::OsError;

use crate::process::{Process, ProcessId, State};
use crate::traps::TrapFrame;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SchedulerError {
    FailedToAllocateProcessId,
    ProcessNotFound,
    NoRunnableProcess,
}

impl From<SchedulerError> for OsError {
    fn from(_: SchedulerError) -> Self {
        OsError::SchedulerError
    }
}

pub type SchedulerResult<T> = Result<T, SchedulerError>;

pub trait Scheduler: Send {
    fn new() -> Self
    where
        Self: Sized;
    fn setup_core(&mut self, core: usize) -> SchedulerResult<()>;

    fn add(&mut self, process: Process) -> SchedulerResult<()>;
    #[deprecated]
    fn remove(&mut self, trap_frame: &mut TrapFrame) -> SchedulerResult<Process>;

    fn switch(
        &mut self,
        trap_frame: &mut TrapFrame,
        trigger: SwitchTrigger,
        state: State,
    ) -> SchedulerResult<()>;
    fn schedule_in(&mut self, trap_frame: &mut TrapFrame) -> SchedulerResult<ProcessId>;

    fn on_process<F, R>(&mut self, trap_frame: &mut TrapFrame, function: F) -> SchedulerResult<R>
        where
            F: FnOnce(&mut Process) -> R,
            Self: Sized;
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum SwitchTrigger {
    Force,
    Yield,
    Timer,
}

impl Into<usize> for SwitchTrigger {
    fn into(self) -> usize {
        match self {
            SwitchTrigger::Force => 0,
            SwitchTrigger::Yield => 1,
            SwitchTrigger::Timer => 2,
        }
    }
}

pub struct SwitchCondition([bool; 3]);

impl SwitchCondition {
    pub fn new() -> Self {
        Self([false, false, false])
    }

    pub fn or(mut self, switch_trigger: SwitchTrigger) -> Self {
        let i: usize = switch_trigger.into();
        self.0[i] = true;
        self
    }

    pub fn matches(&self, switch_trigger: SwitchTrigger) -> bool {
        let i: usize = switch_trigger.into();
        self.0[i]
    }
}

impl fmt::Debug for SwitchCondition {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut r = f.debug_struct("SwitchCondition");

        r.field("Timer", &self.0[Into::<usize>::into(SwitchTrigger::Timer)]);
        r.field("Yield", &self.0[Into::<usize>::into(SwitchTrigger::Yield)]);
        r.field("Force", &self.0[Into::<usize>::into(SwitchTrigger::Force)]);

        r.finish()
    }
}
