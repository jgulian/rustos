use crate::process::{Process, ProcessId};
use crate::traps::TrapFrame;

pub enum SchedulerError {
    FailedToAllocateProcessId,
    ProcessNotFound,
    NoRunnableProcess,
}

pub type SchedulerResult<T> = Result<T, SchedulerError>;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum SwitchTrigger {
    Yield,
    Timer,
}

pub trait Scheduler: Send {
    fn new() -> Self where Self: Sized;

    fn add(&mut self, process: Process) -> SchedulerResult<ProcessId>;
    fn remove(&mut self, trap_frame: &mut TrapFrame) -> SchedulerResult<ProcessId>;
    fn switch(&mut self, trap_frame: &mut TrapFrame, trigger: SwitchTrigger) -> SchedulerResult<()>;
    fn schedule_in(&mut self, trap_frame: &mut TrapFrame) -> SchedulerResult<ProcessId>;

    fn on_process<F, R>(&mut self, id: ProcessId, function: F) -> R
        where F: FnOnce(&mut Process) -> R;
    fn on_process_with_trap_frame<F, R>(&mut self, trap_frame: &mut TrapFrame, function: F) -> R
        where F: FnOnce(&mut Process) -> R {
        let id = ProcessId::from(trap_frame.tpidr);
        self.on_process(id, move |process| {
            *process.context = *trap_frame;
            let result = function(process);
            *trap_frame = *process.context;
            result
        })
    }
}