use core::arch::asm;
use sync::Mutex;

use crate::multiprocessing::spin_lock::SpinLock;
use crate::process::{Process, ProcessId, State};
use crate::scheduling::scheduler::{Scheduler, SchedulerError, SchedulerResult, SwitchTrigger};
use crate::traps::TrapFrame;

extern "C" {
    fn context_restore();
}

pub struct GlobalScheduler<T: Scheduler>(SpinLock<Option<T>>);

impl<T: Scheduler> GlobalScheduler<T> {
    /// Returns an uninitialized wrapper around a local scheduler.
    pub const fn uninitialized() -> Self {
        Self(SpinLock::new(None))
    }

    /// Initializes the scheduler and add userspace processes to the Scheduler.
    pub unsafe fn initialize(&self) {
        self.0.lock(|scheduler| {
            if scheduler.is_some() {
                panic!("scheduler already initialized");
            } else {
                *scheduler = Some(T::new());
            }
        }).expect("failed to lock scheduler");
    }

    pub fn add(&self, process: Process) -> SchedulerResult<ProcessId> {
        let id = self.0.lock(|scheduler| {
            scheduler
                .as_mut()
                .expect("scheduler uninitialized")
                .add(process)
        }).expect("failed to lock scheduler")?;
        aarch64::sev();
        Ok(id)
    }

    pub fn remove(&self, trap_frame: &mut TrapFrame) -> SchedulerResult<Process> {
        self.0.lock(|scheduler| {
            scheduler
                .as_mut()
                .expect("scheduler uninitialized")
                .remove(trap_frame)
        }).expect("failed to lock scheduler")
    }

    pub fn switch(&self, trap_frame: &mut TrapFrame, trigger: SwitchTrigger, state: State) -> SchedulerResult<()> {
        let result = self.0.lock(|scheduler| {
            scheduler
                .as_mut()
                .expect("scheduler uninitialized")
                .switch(trap_frame, trigger, state)
        }).expect("failed to lock scheduler");

        if result.is_ok() || result.is_err_and(|err| err != SchedulerError::NoRunnableProcess) {
            return result;
        }

        aarch64::sev();
        self.schedule_in(trap_frame)
    }

    pub fn schedule_in(&self, trap_frame: &mut TrapFrame) -> SchedulerResult<()> {
        loop {
            let result = self.0.lock(|scheduler| {
                scheduler
                    .as_mut()
                    .expect("scheduler uninitialized")
                    .schedule_in(trap_frame)
            }).expect("failed to lock scheduler").map(|_| ());

            if result.is_ok() || result.is_err_and(|err| err != SchedulerError::NoRunnableProcess) {
                return result;
            }

            aarch64::wfe();
        }
    }

    pub fn on_process<F, R>(&self, trap_frame: &mut TrapFrame, function: F) -> SchedulerResult<R>
        where F: FnOnce(&mut Process) -> R {
        self.0.lock(|scheduler| {
            scheduler
                .as_mut()
                .expect("scheduler uninitialized")
                .on_process(trap_frame, function)
        }).expect("failed to lock scheduler")
    }

    pub fn bootstrap(&self) -> ! {
        self.0.lock(|scheduler| {
            scheduler
                .as_mut()
                .expect("scheduler uninitialized")
                .setup_core(aarch64::affinity())
                .expect("unable to setup core");
        }).expect("failed to lock scheduler");

        let mut trap_frame: TrapFrame = Default::default();
        self.schedule_in(&mut trap_frame).expect("unable to schedule initial process");

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