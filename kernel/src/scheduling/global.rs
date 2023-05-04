use alloc::boxed::Box;
use alloc::collections::btree_map::Entry;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::arch::asm;
use core::time::Duration;

use pi::local_interrupt::{local_tick_in, LocalController, LocalInterrupt};
use pi::timer::current_time;
use sync::Mutex;

use crate::multiprocessing::per_core::local_irq;
use crate::multiprocessing::spin_lock::SpinLock;
use crate::param::TICK;
use crate::process::{Process, ProcessId, State};
use crate::SCHEDULER;
use crate::scheduling::fair_policy::FairPolicy;
use crate::scheduling::policy::{Policy, PolicyAdvice, PolicyInformation};
use crate::scheduling::round_robin_policy::RoundRobinPolicy;
use crate::scheduling::scheduler::{
    Scheduler, SchedulerError, SchedulerResult, SwitchCondition, SwitchTrigger,
};
use crate::traps::irq::IrqHandlerRegistry;
use crate::traps::TrapFrame;

extern "C" {
    fn context_restore();
}

type RunningProcess = (Process, SwitchCondition, Duration);

struct SchedulerInformation {
    policies: Vec<(String, Box<dyn Policy>)>,
    active_policy: usize,
    last_process_id: Option<ProcessId>,
    processes: BTreeMap<ProcessId, Process>,
    running: BTreeMap<ProcessId, RunningProcess>,
}

impl SchedulerInformation {
    fn allocate_process_id(&mut self) -> ProcessId {
        self.last_process_id = Some(
            self.last_process_id
                .map(|pid| ProcessId::from(Into::<u64>::into(pid) + 1))
                .unwrap_or(ProcessId::from(0)),
        );
        self.last_process_id.unwrap()
    }

    fn add(&mut self, mut process: Process) -> ProcessId {
        let process_id = self.allocate_process_id();
        process.context.tpidr = process_id.into();
        self.processes.insert(process_id, process);
        self.policies
            .iter_mut()
            .for_each(|(_, policy)| policy.insert(process_id));
        process_id
    }

    fn schedule_out(
        &mut self,
        trap_frame: &mut TrapFrame,
        trigger: SwitchTrigger,
        state: State,
    ) -> SchedulerResult<bool> {
        let process_id = ProcessId::from(trap_frame.tpidr);
        match self.running.entry(process_id) {
            Entry::Vacant(_) => panic!("attempted switch of non-running process"),
            Entry::Occupied(occupied) => {
                if !occupied.get().1.matches(trigger) {
                    if matches!(trigger, SwitchTrigger::Timer) {
                        local_tick_in(aarch64::affinity(), Duration::from_secs(20));
                    }

                    return Ok(false);
                }

                let (mut process, _, start_time) = occupied.remove();
                *process.context = *trap_frame;

                let policy_information = PolicyInformation::ScheduledOut(
                    current_time() - start_time,
                    trigger,
                    matches!(state, State::Waiting(_)),
                );

                self.on_policies(move |policy| policy.inform(process_id, &policy_information));

                process.state = state;
                match &process.state {
                    State::Ready | State::Waiting(_) => {
                        self.processes.insert(process_id, process);
                    }
                    State::Running => {
                        panic!("can't switch out a process with the running state");
                    }
                    State::Dead => {
                        self.remove_process_from_policies(process_id);
                        if let Some(parent_id) = &process.parent {
                            if let Some(parent) = self.processes.get_mut(parent_id) {
                                parent.dead_children.push(process_id);
                            }
                        }
                    }
                }

                Ok(true)
            }
        }
    }

    fn update_process_statuses(&mut self) {
        for (_, process) in self.processes.iter_mut() {
            if process.done_waiting() {
                let info = PolicyInformation::DoneWaiting;
                for (_, policy) in self.policies.iter_mut() {
                    policy.inform(process.id(), &info)
                }
            }
        }
    }

    fn schedule_in(&mut self, trap_frame: &mut TrapFrame) -> bool {
        if let Some(PolicyAdvice {
                        process_id,
                        runtime,
                        mut stop_condition,
                    }) = self.policies[self.active_policy].1.advise()
        {
            let process = self
                .processes
                .remove(&process_id)
                .expect("attempted to schedule non existent process");

            *trap_frame = *process.context;

            if let Some(tick_in) = runtime {
                local_tick_in(aarch64::affinity(), tick_in);
            } else {
                stop_condition = stop_condition.or(SwitchTrigger::Yield);
            }

            stop_condition = stop_condition.or(SwitchTrigger::Force);

            self.running
                .insert(process_id, (process, stop_condition, current_time()));
            let policy_information = PolicyInformation::StartRunning;
            self.on_policies(move |policy| policy.inform(process_id, &policy_information));

            true
        } else {
            false
        }
    }

    fn remove_process_from_policies(&mut self, process_id: ProcessId) {
        self.policies
            .iter_mut()
            .for_each(|(_, policy)| policy.remove(process_id));
    }

    fn on_policies<F: Fn(&mut Box<dyn Policy>) + 'static>(&mut self, function: F) {
        for (_, policy) in self.policies.iter_mut() {
            function(policy)
        }
    }
}

pub struct GlobalScheduler(SpinLock<Option<SchedulerInformation>>);

impl GlobalScheduler {
    /// Returns an uninitialized wrapper around a local scheduler.
    pub const fn uninitialized() -> Self {
        Self(SpinLock::new(None))
    }

    /// Initializes the scheduler and add userspace processes to the Scheduler.
    pub unsafe fn initialize(&self) {
        self.0
            .lock(|scheduler| {
                if scheduler.is_some() {
                    panic!("scheduler already initialized");
                } else {
                    *scheduler = Some(SchedulerInformation {
                        policies: vec![
                            (
                                String::from("RoundRobin"),
                                Box::new(RoundRobinPolicy::new(TICK)),
                            ),
                            (
                                String::from("Fair"),
                                Box::new(FairPolicy::new(
                                    Duration::from_millis(48),
                                    Duration::from_millis(6),
                                )),
                            ),
                        ],
                        active_policy: 1,
                        last_process_id: None,
                        processes: BTreeMap::new(),
                        running: BTreeMap::new(),
                    })
                }
            })
            .expect("failed to lock scheduler");
    }

    pub fn add(&self, process: Process) -> SchedulerResult<ProcessId> {
        let process_id = self
            .0
            .lock(|scheduler| {
                scheduler
                    .as_mut()
                    .map(|scheduler_information| Ok(scheduler_information.add(process)))
                    .expect("scheduler uninitialized")
            })
            .expect("failed to lock scheduler")?;
        aarch64::sev();
        Ok(process_id)
    }

    pub fn switch(
        &self,
        trap_frame: &mut TrapFrame,
        trigger: SwitchTrigger,
        state: State,
    ) -> SchedulerResult<()> {
        let scheduled_out = self
            .0
            .lock(|scheduler| {
                scheduler
                    .as_mut()
                    .map(|scheduler_information| {
                        scheduler_information.schedule_out(trap_frame, trigger, state)
                    })
                    .expect("scheduler uninitialized")
            })
            .expect("failed to lock scheduler")?;

        if scheduled_out {
            aarch64::sev();
            self.schedule_in(trap_frame)
        } else {
            Ok(())
        }
    }

    pub fn schedule_in(&self, trap_frame: &mut TrapFrame) -> SchedulerResult<()> {
        while {
            let scheduled = self
                .0
                .lock(|scheduler| {
                    scheduler
                        .as_mut()
                        .map(|scheduler_information| {
                            scheduler_information.update_process_statuses();
                            scheduler_information.schedule_in(trap_frame)
                        })
                        .expect("scheduler uninitialized")
                })
                .expect("failed to lock scheduler");

            aarch64::wfe();

            !scheduled
        } {}

        Ok(())
    }

    pub fn on_process<F, R>(&self, trap_frame: &mut TrapFrame, function: F) -> SchedulerResult<R>
    where
        F: FnOnce(&mut Process) -> R,
    {
        self.0
            .lock(|scheduler| {
                scheduler
                    .as_mut()
                    .map(|scheduler_information| {
                        let process = scheduler_information
                            .processes
                            .iter_mut()
                            .map(|(_, process)| process)
                            .chain(
                                scheduler_information
                                    .running
                                    .iter_mut()
                                    .map(|(_, (process, _, _))| process),
                            )
                            .find(|process| process.context.tpidr == trap_frame.tpidr)
                            .ok_or(SchedulerError::ProcessNotFound)?;
                        *process.context = *trap_frame;
                        let result = function(process);
                        *trap_frame = *process.context;
                        Ok(result)
                    })
                    .expect("scheduler uninitialized")
            })
            .expect("failed to lock scheduler")
    }

    pub fn bootstrap(&self) -> ! {
        let mut controller = LocalController::new(aarch64::affinity());
        controller.enable_local_timer();

        local_irq().register(
            LocalInterrupt::CntPnsIrq,
            Box::new(|trap_frame| {
                //kprintln!("timer went off");
                let core = aarch64::affinity();
                SCHEDULER
                    .switch(trap_frame, SwitchTrigger::Timer, State::Ready)
                    .expect("failed to switch processes");
            }),
        );
        local_tick_in(aarch64::affinity(), TICK);

        let mut trap_frame: TrapFrame = Default::default();
        self.schedule_in(&mut trap_frame)
            .expect("unable to schedule initial process");

        //kprintln!("scheduled in {:?}", trap_frame);

        unsafe {
            asm!(
            "mov sp, {stack}",
            "bl {context_restore}",
            "ldp x28, x29, [SP], #16",
            "ldp lr, xzr, [SP], #16",
            "eret",
            stack = in(reg) (&mut trap_frame) as *const TrapFrame as u64,
            context_restore = sym context_restore,
            );
        }

        panic!("unable to bootstrap to userspace");
    }

    pub fn set_active_scheduler(&self, active_policy: usize) {
        self.0
            .lock(|scheduler| {
                scheduler
                    .as_mut()
                    .map(|scheduler_information| {
                        scheduler_information.active_policy = active_policy;
                    })
                    .expect("scheduler uninitialized")
            })
            .expect("failed to lock scheduler");
    }
}
