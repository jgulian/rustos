use alloc::vec::Vec;
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

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum SwitchTrigger {
    Force,
    Yield,
    Timer,
}

pub trait Scheduler: From<Vec<Process>> + Into<Vec<Process>> + Send {
    fn new() -> Self where Self: Sized;
    fn setup_core(&mut self, core: usize) -> SchedulerResult<()>;

    fn add(&mut self, process: Process) -> SchedulerResult<ProcessId>;
    fn remove(&mut self, trap_frame: &mut TrapFrame) -> SchedulerResult<Process>;

    fn switch(&mut self, trap_frame: &mut TrapFrame, trigger: SwitchTrigger, state: State) -> SchedulerResult<()>;
    fn schedule_in(&mut self, trap_frame: &mut TrapFrame) -> SchedulerResult<ProcessId>;

    fn on_process<F, R>(&mut self, trap_frame: &mut TrapFrame, function: F) -> SchedulerResult<R>
        where F: FnOnce(&mut Process) -> R, Self: Sized;
}