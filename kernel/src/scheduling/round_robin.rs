use alloc::boxed::Box;
use alloc::collections::VecDeque;
use core::fmt;
use pi::local_interrupt::{local_tick_in, LocalController, LocalInterrupt};
use crate::multiprocessing::per_core::local_irq;
use crate::param::TICK;
use crate::process::{Process, ProcessId, State};
use crate::SCHEDULER;
use crate::scheduling::scheduler::{Scheduler, SchedulerError, SchedulerResult, SwitchTrigger};
use crate::scheduling::scheduler::SchedulerError::{FailedToAllocateProcessId, ProcessNotFound};
use crate::traps::irq::IrqHandlerRegistry;
use crate::traps::TrapFrame;

/// Internal scheduler struct which is not thread-safe.
pub struct RoundRobinScheduler {
    processes: VecDeque<Process>,
    last_id: Option<ProcessId>,
}

impl RoundRobinScheduler {
    fn allocate_process_id(&mut self) -> Option<ProcessId> {
        let next_id = match self.last_id {
            None => 0u64,
            Some(pid) => {
                if pid == u64::MAX {
                    return None;
                }
                pid.into() + 1
            }
        };

        self.last_id = Some(ProcessId::from(next_id));
        self.last_id
    }

    /// Finds the currently running process, sets the current process's state
    /// to `new_state`, prepares the context switch on `tf` by saving `tf`
    /// into the current process, and push the current process back to the
    /// end of `processes` queue.
    ///
    /// If the `processes` queue is empty or there is no current process,
    /// returns `false`. Otherwise, returns `true`.
    fn schedule_out(&mut self, state: State, trap_frame: &mut TrapFrame) -> bool {
        let found_process = self.processes.iter_mut()
            .enumerate()
            .filter_map(|(i, process)|
                if process.context.tpidr == trap_frame.tpidr { Some(i) } else { None })
            .find();

        if let Some(i) = found_process {
            let mut process = self.processes.remove(i).unwrap();
            process.state = state;
            *process.context = *trap_frame;
            self.processes.push_back(process);

            true
        } else {
            false
        }
    }
}

impl Scheduler for RoundRobinScheduler {
    fn new() -> Self where Self: Sized {
        initialize_local_timer_interrupt();
        let core = aarch64::affinity();
        local_tick_in(core, TICK);

        RoundRobinScheduler {
            processes: Default::default(),
            last_id: None,
        }
    }

    fn add(&mut self, mut process: Process) -> SchedulerResult<ProcessId> {
        let new_pid = self.allocate_process_id()
            .ok_or(FailedToAllocateProcessId)?;

        process.context.tpidr = new_pid.into();
        self.processes.push_back(process);

        self.last_id = Some(new_pid);
        Ok(new_pid)
    }

    fn remove(&mut self, trap_frame: &mut TrapFrame) -> SchedulerResult<ProcessId> {
        self.schedule_out(State::Dead, trap_frame);
        let process = self.processes.pop_back().ok_or(ProcessNotFound)?;
        let process_id = ProcessId::from(trap_frame.tpidr);

        if let Some(parent_id) = process.parent {
            self.processes.iter_mut()
                .filter(|process| process.id() == parent_id)
                .for_each(|process| process.dead_children.push(process.id()));
        }

        self.schedule_in(trap_frame)?;

        Ok(process_id)
    }

    fn switch(&mut self, trap_frame: &mut TrapFrame, trigger: SwitchTrigger) -> SchedulerResult<()> {
        if trigger == SwitchTrigger::Timer {
            self.schedule_out(State::Ready, trap_frame);
            self.schedule_in(trap_frame)?;
        }

        Ok(())
    }

    fn schedule_in(&mut self, trap_frame: &mut TrapFrame) -> SchedulerResult<ProcessId> {
        let runnable_process = self.processes.iter_mut()
            .enumerate()
            .filter_map(|(i, process)| if process.can_run() {Some(i)} else {None})
            .next().ok_or(SchedulerError::NoRunnableProcess)?;

        let mut process = self.processes.remove(runnable_process).unwrap();
        let id = process.id();
        process.state = State::Running;
        *trap_frame = *process.context;
        self.processes.push_front(process);
        Ok(id)
    }

    fn on_process<F, R>(&mut self, id: ProcessId, function: F) -> R where F: FnOnce(&mut Process) -> R {
        todo!()
    }
}

impl fmt::Debug for RoundRobinScheduler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let len = self.processes.len();
        writeln!(f, "  [Scheduler] {} processes in the queue", len)?;
        for i in 0..len {
            writeln!(
                f,
                "    queue[{}]: proc({:3})-{:?} ",
                i, self.processes[i].context.tpidr, self.processes[i].state
            )?;
        }
        Ok(())
    }
}

pub fn initialize_local_timer_interrupt() {
    let core = aarch64::affinity();
    let mut controller = LocalController::new(core);
    controller.enable_local_timer();

    local_irq().register(LocalInterrupt::CntPnsIrq, Box::new(|trap_frame| {
        let core = aarch64::affinity();
        SCHEDULER.switch(trap_frame, SwitchTrigger::Timer);
        local_tick_in(core, TICK);
    }));
    local_tick_in(core, TICK);
}