use alloc::boxed::Box;
use alloc::collections::BinaryHeap;
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::time::Duration;
use pi::local_interrupt::{local_tick_in, LocalController, LocalInterrupt};
use crate::multiprocessing::per_core::local_irq;
use crate::process::{Process, ProcessId, State};
use crate::SCHEDULER;
use crate::scheduling::{Scheduler, SwitchTrigger};
use crate::scheduling::scheduler::SchedulerResult;
use crate::traps::irq::IrqHandlerRegistry;
use crate::traps::TrapFrame;

pub struct ProportionalShareScheduler {
    processes: BinaryHeap<ProcessInformation>,
    scheduler_latency: Duration,
    minimum_granularity: Duration,
}

impl Scheduler for ProportionalShareScheduler {
    fn new() -> Self where Self: Sized {
        Self {
            processes: BinaryHeap::default(),
            scheduler_latency: Duration::from_millis(48),
            minimum_granularity: Duration::from_millis(6),
        }
    }

    fn setup_core(&mut self, core: usize) -> SchedulerResult<()> {
        let mut controller = LocalController::new(core);
        controller.enable_local_timer();

        local_irq().register(LocalInterrupt::CntPnsIrq, Box::new(|trap_frame| {
            SCHEDULER.switch(trap_frame, SwitchTrigger::Timer, State::Ready)
                .expect("failed to switch processes");
        }));

        Ok(())
    }

    fn add(&mut self, process: Process) -> SchedulerResult<ProcessId> {
        todo!()
    }

    fn remove(&mut self, trap_frame: &mut TrapFrame) -> SchedulerResult<Process> {
        todo!()
    }

    fn switch(&mut self, trap_frame: &mut TrapFrame, trigger: SwitchTrigger, state: State) -> SchedulerResult<()> {
        todo!()
    }

    fn schedule_in(&mut self, trap_frame: &mut TrapFrame) -> SchedulerResult<ProcessId> {

        //local_tick_in(core, TICK);
        todo!()
    }

    fn on_process<F, R>(&mut self, trap_frame: &mut TrapFrame, function: F) -> SchedulerResult<R> where F: FnOnce(&mut Process) -> R, Self: Sized {
        todo!()
    }
}

impl From<Vec<Process>> for ProportionalShareScheduler {
    fn from(value: Vec<Process>) -> Self {
        todo!()
    }
}

impl Into<Vec<Process>> for ProportionalShareScheduler {
    fn into(self) -> Vec<Process> {
        todo!()
    }
}

struct ProcessInformation {
    process: Process,
    virtual_runtime: usize,
}

impl Eq for ProcessInformation {}

impl PartialEq<Self> for ProcessInformation {
    fn eq(&self, other: &Self) -> bool {
        self.virtual_runtime == other.virtual_runtime
    }
}

impl PartialOrd<Self> for ProcessInformation {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.virtual_runtime.partial_cmp(&other.virtual_runtime)
    }
}

impl Ord for ProcessInformation {
    fn cmp(&self, other: &Self) -> Ordering {
        self.virtual_runtime.cmp(&other.virtual_runtime)
    }
}