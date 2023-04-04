

use core::arch::asm;
use core::ops::Add;






use sync::Mutex;

use crate::multiprocessing::spin_lock::SpinLock;
use crate::process::{Process, ProcessId, State};
use crate::scheduling::proportional_share::ProportionalShareScheduler;
use crate::scheduling::scheduler::{Scheduler, SchedulerError, SchedulerResult, SwitchTrigger};
use crate::scheduling::RoundRobinScheduler;
use crate::traps::TrapFrame;


extern "C" {
    fn context_restore();
}

enum Schedulers {
    ProportionalShare(ProportionalShareScheduler),
    RoundRobin(RoundRobinScheduler),
}

impl Schedulers {
    fn get_scheduler(&mut self) -> &mut dyn Scheduler {
        match self {
            Schedulers::ProportionalShare(proportional_share) => proportional_share,
            Schedulers::RoundRobin(round_robin) => round_robin,
        }
    }
}

struct SchedulerInformation {
    scheduler: Schedulers,
    last_process_id: Option<ProcessId>,
}

impl SchedulerInformation {
    fn allocate_process_id(&mut self) -> ProcessId {
        let next_id = match self.last_process_id {
            None => ProcessId::from(0u64),
            Some(pid) => {
                //if pid == ProcessId::from(u64::MAX) {
                //    return None;
                //}
                let raw: u64 = pid.into();
                ProcessId::from(raw + 1u64)
            }
        };

        self.last_process_id = Some(next_id);
        next_id
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
                        scheduler: Schedulers::ProportionalShare(ProportionalShareScheduler::new()),
                        last_process_id: None,
                    })
                }
            })
            .expect("failed to lock scheduler");
    }

    pub fn add(&self, mut process: Process) -> SchedulerResult<ProcessId> {
        let id = self
            .0
            .lock(|scheduler| {
                let scheduler_information = scheduler.as_mut().expect("scheduler uninitialized");
                let process_id = scheduler_information.allocate_process_id();
                process.context.tpidr = process_id.into();
                scheduler_information
                    .scheduler
                    .get_scheduler()
                    .add(process)?;
                Ok(process_id)
            })
            .expect("failed to lock scheduler")?;
        aarch64::sev();
        Ok(id)
    }

    #[deprecated]
    fn remove(&self, trap_frame: &mut TrapFrame) -> SchedulerResult<Process> {
        self.0
            .lock(|scheduler| {
                scheduler
                    .as_mut()
                    .expect("scheduler uninitialized")
                    .scheduler
                    .get_scheduler()
                    .remove(trap_frame)
            })
            .expect("failed to lock scheduler")
    }

    pub fn switch(
        &self,
        trap_frame: &mut TrapFrame,
        trigger: SwitchTrigger,
        state: State,
    ) -> SchedulerResult<()> {
        let result = self
            .0
            .lock(|scheduler| {
                scheduler
                    .as_mut()
                    .expect("scheduler uninitialized")
                    .scheduler
                    .get_scheduler()
                    .switch(trap_frame, trigger, state)
            })
            .expect("failed to lock scheduler");

        if result.is_ok() || result.is_err_and(|err| err != SchedulerError::NoRunnableProcess) {
            return result;
        }

        aarch64::sev();
        self.schedule_in(trap_frame)
    }

    pub fn schedule_in(&self, trap_frame: &mut TrapFrame) -> SchedulerResult<()> {
        loop {
            let result = self
                .0
                .lock(|scheduler| {
                    scheduler
                        .as_mut()
                        .expect("scheduler uninitialized")
                        .scheduler
                        .get_scheduler()
                        .schedule_in(trap_frame)
                })
                .expect("failed to lock scheduler")
                .map(|_| ());

            if result.is_ok() || result.is_err_and(|err| err != SchedulerError::NoRunnableProcess) {
                return result;
            }

            aarch64::wfe();
        }
    }

    pub fn on_process<F, R>(&self, trap_frame: &mut TrapFrame, function: F) -> SchedulerResult<R>
    where
        F: FnOnce(&mut Process) -> R,
    {
        self.0
            .lock(|scheduler| {
                let scheduler_information = scheduler.as_mut().expect("scheduler uninitialized");
                match &mut scheduler_information.scheduler {
                    Schedulers::ProportionalShare(proportional_share) => {
                        proportional_share.on_process(trap_frame, function)
                    }
                    Schedulers::RoundRobin(round_robin) => {
                        round_robin.on_process(trap_frame, function)
                    }
                }
            })
            .expect("failed to lock scheduler")
    }

    pub fn bootstrap(&self) -> ! {
        self.0
            .lock(|scheduler| {
                scheduler
                    .as_mut()
                    .expect("scheduler uninitialized")
                    .scheduler
                    .get_scheduler()
                    .setup_core(aarch64::affinity())
                    .expect("unable to setup core");
            })
            .expect("failed to lock scheduler");

        let mut trap_frame: TrapFrame = Default::default();
        self.schedule_in(&mut trap_frame)
            .expect("unable to schedule initial process");r

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
}
