use alloc::boxed::Box;
use alloc::collections::VecDeque;
use core::fmt;

use pi::local_interrupt::{local_tick_in, LocalController, LocalInterrupt};

use crate::multiprocessing::per_core::local_irq;
use crate::param::TICK;
use crate::process::{Process, ProcessId, State};
use crate::SCHEDULER;
use crate::scheduling::scheduler::{Scheduler, SchedulerError, SchedulerResult, SwitchTrigger};
use crate::traps::irq::IrqHandlerRegistry;
use crate::traps::TrapFrame;

/// Internal scheduler struct which is not thread-safe.
pub struct RoundRobinScheduler {
    processes: VecDeque<Process>,
}

impl RoundRobinScheduler {
    /// Finds the currently running process, sets the current process's state
    /// to `new_state`, prepares the context switch on `tf` by saving `tf`
    /// into the current process, and push the current process back to the
    /// end of `processes` queue.
    ///
    /// If the `processes` queue is empty or there is no current process,
    /// returns `false`. Otherwise, returns `true`.
    fn schedule_out(&mut self, trap_frame: &mut TrapFrame, state: State) -> SchedulerResult<()> {
        let found_process = self
            .processes
            .iter_mut()
            .enumerate()
            .filter_map(|(i, process)| {
                if process.context.tpidr == trap_frame.tpidr {
                    Some(i)
                } else {
                    None
                }
            })
            .next()
            .ok_or(SchedulerError::ProcessNotFound)?;

        let mut process = self.processes.remove(found_process).unwrap();
        process.state = state;
        *process.context = *trap_frame;
        self.processes.push_back(process);

        Ok(())
    }
}

impl Scheduler for RoundRobinScheduler {
    fn new() -> Self
    where
        Self: Sized,
    {
        RoundRobinScheduler {
            processes: Default::default(),
        }
    }

    fn setup_core(&mut self, core: usize) -> SchedulerResult<()> {
        let mut controller = LocalController::new(core);
        controller.enable_local_timer();

        local_irq().register(
            LocalInterrupt::CntPnsIrq,
            Box::new(|trap_frame| {
                let core = aarch64::affinity();
                SCHEDULER
                    .switch(trap_frame, SwitchTrigger::Timer, State::Ready)
                    .expect("failed to switch processes");
                local_tick_in(core, TICK);
            }),
        );
        local_tick_in(core, TICK);

        Ok(())
    }

    fn add(&mut self, process: Process) -> SchedulerResult<()> {
        self.processes.push_back(process);
        Ok(())
    }

    fn remove(&mut self, trap_frame: &mut TrapFrame) -> SchedulerResult<Process> {
        self.schedule_out(trap_frame, State::Ready)?;
        let process = self.processes.pop_back().unwrap();
        self.schedule_in(trap_frame)?;

        Ok(process)
    }

    fn switch(
        &mut self,
        trap_frame: &mut TrapFrame,
        trigger: SwitchTrigger,
        state: State,
    ) -> SchedulerResult<()> {
        if matches!(trigger, SwitchTrigger::Force | SwitchTrigger::Timer) {
            let is_dying = matches!(state, State::Dead);
            self.schedule_out(trap_frame, state)?;

            if is_dying {
                let process = self
                    .processes
                    .pop_back()
                    .ok_or(SchedulerError::ProcessNotFound)?;

                if let Some(parent_id) = process.parent {
                    self.processes
                        .iter_mut()
                        .filter(|process| process.id() == parent_id)
                        .for_each(|process| process.dead_children.push(process.id()));
                }
            }

            self.schedule_in(trap_frame)?;
        }

        Ok(())
    }

    fn schedule_in(&mut self, trap_frame: &mut TrapFrame) -> SchedulerResult<ProcessId> {
        let runnable_process = self
            .processes
            .iter_mut()
            .enumerate()
            .find_map(|(i, process)| if process.can_run() { Some(i) } else { None })
            .ok_or(SchedulerError::NoRunnableProcess)?;

        let mut process = self.processes.remove(runnable_process).unwrap();
        let id = process.id();
        process.state = State::Running;
        *trap_frame = *process.context;
        self.processes.push_front(process);
        Ok(id)
    }

    fn on_process<F, R>(&mut self, trap_frame: &mut TrapFrame, function: F) -> SchedulerResult<R>
    where
        F: FnOnce(&mut Process) -> R,
        Self: Sized,
    {
        let process = self
            .processes
            .iter_mut().find(|process| process.id() == ProcessId::from(trap_frame.tpidr))
            .ok_or(SchedulerError::ProcessNotFound)?;

        *process.context = *trap_frame;
        let result = function(process);
        *trap_frame = *process.context;

        Ok(result)
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
