use crate::multiprocessing::per_core::local_irq;
use crate::process::{Process, ProcessId, State};
use crate::scheduling::scheduler::{SchedulerError, SchedulerResult};
use crate::scheduling::{Scheduler, SwitchTrigger};
use crate::traps::irq::IrqHandlerRegistry;
use crate::traps::TrapFrame;
use crate::SCHEDULER;
use alloc::boxed::Box;

use alloc::vec::Vec;
use core::cmp;
use core::cmp::Ordering;
use core::time::Duration;
use pi::local_interrupt::{local_tick_in, LocalController, LocalInterrupt};

pub struct ProportionalShareScheduler {
    running: Vec<(ProcessInformation, Duration)>,
    processes: Vec<ProcessInformation>,
    scheduler_latency: Duration,
    minimum_granularity: Duration,
}

impl ProportionalShareScheduler {
    fn schedule_out(
        &mut self,
        trap_frame: &mut TrapFrame,
        state: State,
    ) -> SchedulerResult<ProcessInformation> {
        let process_index = self
            .running
            .iter()
            .enumerate()
            .find_map(|(i, (process_information, _))| {
                if process_information.process.context.tpidr == trap_frame.tpidr {
                    Some(i)
                } else {
                    None
                }
            })
            .ok_or(SchedulerError::ProcessNotFound)?;

        let (mut process_information, scheduled_in_at) = self.running.remove(process_index);
        process_information.process.state = state;
        *process_information.process.context = *trap_frame;
        let current_time = pi::timer::current_time();
        process_information.virtual_runtime += (current_time - scheduled_in_at).as_micros();
        Ok(process_information)
    }

    fn all_processes(&mut self) -> impl Iterator<Item = &mut ProcessInformation> {
        self.running
            .iter_mut()
            .map(|(p, _)| p)
            .chain(self.processes.iter_mut())
    }
}

impl Scheduler for ProportionalShareScheduler {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            running: Vec::default(),
            processes: Vec::default(),
            scheduler_latency: Duration::from_millis(48),
            minimum_granularity: Duration::from_millis(6),
        }
    }

    fn setup_core(&mut self, core: usize) -> SchedulerResult<()> {
        let mut controller = LocalController::new(core);
        controller.enable_local_timer();

        local_irq().register(
            LocalInterrupt::CntPnsIrq,
            Box::new(|trap_frame| {
                SCHEDULER
                    .switch(trap_frame, SwitchTrigger::Timer, State::Ready)
                    .expect("failed to switch processes");
            }),
        );

        Ok(())
    }

    fn add(&mut self, process: Process) -> SchedulerResult<()> {
        self.processes.push(ProcessInformation {
            process,
            virtual_runtime: 0,
        });
        Ok(())
    }

    fn remove(&mut self, trap_frame: &mut TrapFrame) -> SchedulerResult<Process> {
        let process_information = self.schedule_out(trap_frame, State::Dead)?;
        self.schedule_in(trap_frame)?;
        Ok(process_information.process)
    }

    fn switch(
        &mut self,
        trap_frame: &mut TrapFrame,
        _: SwitchTrigger,
        state: State,
    ) -> SchedulerResult<()> {
        let is_dying = state == State::Dead;
        let process_information = self.schedule_out(trap_frame, state)?;

        if is_dying {
            if let Some(parent_id) = process_information.process.parent {
                self.all_processes()
                    .map(|process_information| &mut process_information.process)
                    .filter(|process| process.id() == parent_id)
                    .for_each(|process| process.dead_children.push(process.id()));
            }
            let _ = process_information;
        } else {
            self.processes.push(process_information);
        }

        self.schedule_in(trap_frame)?;
        Ok(())
    }

    fn schedule_in(&mut self, trap_frame: &mut TrapFrame) -> SchedulerResult<ProcessId> {
        let (index, _) = self
            .processes
            .iter_mut()
            .enumerate()
            .fold(None, |result, (i, process_information)| {
                if !process_information.process.can_run() {
                    result
                } else {
                    match result {
                        None => Some((i, process_information.virtual_runtime)),
                        Some((index, virtual_runtime)) => {
                            if process_information.virtual_runtime > virtual_runtime {
                                Some((i, process_information.virtual_runtime))
                            } else {
                                Some((index, virtual_runtime))
                            }
                        }
                    }
                }
            })
            .ok_or(SchedulerError::NoRunnableProcess)?;

        let mut process_information = self.processes.remove(index);
        process_information.process.state = State::Running;
        let process_id = process_information.process.id();
        *trap_frame = *process_information.process.context;
        //info!("scheduled in {} with {:x?} {:x?}", process_information.process.context.tpidr, process_information.process.context.ttbr0, process_information.process.context.ttbr1);
        self.running
            .push((process_information, pi::timer::current_time()));

        let scheduler_latency = self
            .scheduler_latency
            .checked_div(self.processes.len() as u32)
            .unwrap_or(self.scheduler_latency);
        let time_slice = cmp::max(scheduler_latency, self.minimum_granularity);
        local_tick_in(aarch64::affinity(), time_slice);

        Ok(process_id)
    }

    fn on_process<F, R>(&mut self, trap_frame: &mut TrapFrame, function: F) -> SchedulerResult<R>
    where
        F: FnOnce(&mut Process) -> R,
        Self: Sized,
    {
        let process = self
            .all_processes()
            .find_map(|process_information| {
                if process_information.process.context.tpidr == trap_frame.tpidr {
                    Some(&mut process_information.process)
                } else {
                    None
                }
            })
            .ok_or(SchedulerError::ProcessNotFound)?;

        *process.context = *trap_frame;
        let result = function(process);
        *trap_frame = *process.context;

        Ok(result)
    }
}

impl From<Vec<Process>> for ProportionalShareScheduler {
    fn from(_value: Vec<Process>) -> Self {
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
    virtual_runtime: u128,
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
