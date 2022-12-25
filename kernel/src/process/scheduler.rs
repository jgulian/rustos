use alloc::boxed::Box;
use alloc::collections::vec_deque::VecDeque;

use core::arch::asm;
use core::fmt;


use aarch64;
use aarch64::{SP};

use pi::local_interrupt::{local_tick_in, LocalController, LocalInterrupt};
use shim::{io, newioerr};

use crate::{SCHEDULER, VMM};
use crate::multiprocessing::mutex::Mutex;
use crate::multiprocessing::per_core::local_irq;
use crate::param::*;
use crate::process::{Id, Process, State};
use crate::traps::irq::IrqHandlerRegistry;
use crate::traps::TrapFrame;

extern "C" {
    fn _start();
    fn context_restore();
}

/// Process scheduler for the entire machine.
#[derive(Debug)]
pub struct GlobalScheduler(Mutex<Option<Box<Scheduler>>>);

impl GlobalScheduler {
    /// Returns an uninitialized wrapper around a local scheduler.
    pub const fn uninitialized() -> GlobalScheduler {
        GlobalScheduler(Mutex::new(None))
    }

    /// Enters a critical region and execute the provided closure with a mutable
    /// reference to the inner scheduler.
    pub fn critical<F, R>(&self, f: F) -> R
        where
            F: FnOnce(&mut Scheduler) -> R,
    {
        let mut guard = self.0.lock();
        f(guard.as_mut().expect("scheduler uninitialized"))
    }

    /// Adds a process to the scheduler's queue and returns that process's ID.
    /// For more details, see the documentation on `Scheduler::add()`.
    pub fn add(&self, mut process: Process) -> Option<Id> {
        process.context.ttbr0 = VMM.get_baddr().as_u64();
        process.context.ttbr1 = process.vmap.get_baddr().as_u64();
        process.context.elr = USER_IMG_BASE as u64;

        let id = self.critical(move |scheduler| scheduler.add(process));
        aarch64::sev();
        id
    }

    pub fn fork(&self, tf: &mut TrapFrame) -> Option<Id> {
        let id = self.critical(|scheduler| {
            *scheduler.find_process(tf.tpidr)?.context = *tf;
            let id = scheduler.fork(tf.tpidr)?;
            *tf = *scheduler.find_process(tf.tpidr)?.context;
            Some(id)
        })?;

        aarch64::sev();
        Some(id)
    }

    /// Performs a context switch using `tf` by setting the state of the current
    /// process to `new_state`, saving `tf` into the current process, and
    /// restoring the next process's trap frame into `tf`. For more details, see
    /// the documentation on `Scheduler::schedule_out()` and `Scheduler::switch_to()`.
    pub fn switch(&self, new_state: State, tf: &mut TrapFrame) -> Id {
        self.critical(|scheduler| scheduler.schedule_out(new_state, tf));
        aarch64::sev();
        self.switch_to(tf)
    }

    /// Loops until it finds the next process to schedule.
    /// Call `wfi()` in the loop when no process is ready.
    /// For more details, see the documentation on `Scheduler::switch_to()`.
    ///
    /// Returns the process's ID when a ready process is found.
    pub fn switch_to(&self, tf: &mut TrapFrame) -> Id {
        loop {
            let rtn = self.critical(|scheduler| {
                scheduler.switch_to(tf)
            });
            if let Some(id) = rtn {
                trace!(
                    "[core-{}] switch_to {:?}, pc: {:x}, lr: {:x}, x29: {:x}, x28: {:x}, x27: {:x}",
                    aarch64::affinity(),
                    id,
                    tf.elr,
                    tf.xs[30],
                    tf.xs[29],
                    tf.xs[28],
                    tf.xs[27]
                );
                return id;
            }

            aarch64::wfe();
        }
    }

    /// Kills currently running process and returns that process's ID.
    /// For more details, see the documentation on `Scheduler::kill()`.
    #[must_use]
    pub fn kill(&self, tf: &mut TrapFrame) -> Option<Id> {
        self.critical(|scheduler| {
            scheduler.kill(tf)
        })
    }

    /// Starts executing processes in user space using timer interrupt based
    /// preemptive scheduling. This method should not return under normal
    /// conditions.
    pub fn start(&self) -> ! {
        self.initialize_local_timer_interrupt();

        let mut trap_frame: TrapFrame = Default::default();
        self.switch_to(&mut trap_frame);

        unsafe {
            SP.set((&mut trap_frame) as *const TrapFrame as u64);
            context_restore();
            asm!("ldp x28, x29, [SP], #16");
            asm!("ldp lr, xzr, [SP], #16");

            // Todo: Figure out how to reset SP_EL1
        }

        unsafe {
            aarch64::eret();
        }

        loop {}
    }

    /// Initializes the per-core local timer interrupt with `pi::local_interrupt`.
    /// The timer should be configured in a way that `CntpnsIrq` interrupt fires
    /// every `TICK` duration, which is defined in `param.rs`.
    pub fn initialize_local_timer_interrupt(&self) {
        let core = aarch64::affinity();
        let mut controller = LocalController::new(core);
        controller.enable_local_timer();

        local_irq().register(LocalInterrupt::CntPnsIrq, Box::new(|tf| {
            let core = aarch64::affinity();
            SCHEDULER.switch(State::Ready, tf);
            local_tick_in(core, TICK);
        }));
        local_tick_in(core, TICK);
    }

    /// Initializes the scheduler and add userspace processes to the Scheduler.
    pub unsafe fn initialize(&self) {
        *self.0.lock() = Some(Scheduler::new());
    }

    pub fn on_process<T: FnOnce(&mut Process) -> R, R>(&self, tf: &mut TrapFrame, on: T) -> io::Result<R> {
        self.critical(|scheduler| -> io::Result<R> {
            let process = scheduler.find_process(tf.tpidr)
                .ok_or(newioerr!(NotFound))?;
            *process.context = *tf;
            let result = on(process);
            *tf = *process.context;

            Ok(result)
        })
    }
}

/// Internal scheduler struct which is not thread-safe.
pub struct Scheduler {
    processes: VecDeque<Process>,
    last_id: Option<Id>,
}

impl Scheduler {
    /// Returns a new `Scheduler` with an empty queue.
    fn new() -> Box<Scheduler> {
        Box::new(Scheduler {
            processes: VecDeque::new(),
            last_id: None,
        })
    }

    /// Adds a process to the scheduler's queue and returns that process's ID if
    /// a new process can be scheduled. The process ID is newly allocated for
    /// the process and saved in its `trap_frame`. If no further processes can
    /// be scheduled, returns `None`.
    ///
    /// It is the caller's responsibility to ensure that the first time `switch`
    /// is called, that process is executing on the CPU.
    fn add(&mut self, mut process: Process) -> Option<Id> {
        let new_pid = self.new_pid()?;

        (*process.context).tpidr = new_pid;
        self.processes.push_back(process);

        self.last_id = Some(new_pid);
        Some(new_pid)
    }

    fn fork(&mut self, process_id: Id) -> Option<Id> {
        let new_pid = self.new_pid()?;

        let new_process = self.processes.iter_mut()
            .find(|process| process.context.tpidr == process_id)?
            .fork(new_pid).ok()?;

        self.processes.push_back(new_process);
        Some(new_pid)
    }

    fn new_pid(&mut self) -> Option<Id> {
        self.last_id = match self.last_id {
            None => Some(Id::from(0u64)),
            Some(pid) => {
                if pid == u64::MAX {
                    return None;
                }
                Some(Id::from(pid + 1))
            }
        };

        self.last_id
    }

    /// Finds the currently running process, sets the current process's state
    /// to `new_state`, prepares the context switch on `tf` by saving `tf`
    /// into the current process, and push the current process back to the
    /// end of `processes` queue.
    ///
    /// If the `processes` queue is empty or there is no current process,
    /// returns `false`. Otherwise, returns `true`.
    fn schedule_out(&mut self, new_state: State, tf: &mut TrapFrame) -> bool {
        for (i, process) in self.processes.iter_mut().enumerate() {
            if process.context.tpidr == tf.tpidr {
                process.state = new_state;
                *process.context = *tf;
                let p = self.processes.remove(i).unwrap();
                self.processes.push_back(p);

                return true;
            }
        }

        false
    }

    /// Finds the next process to switch to, brings the next process to the
    /// front of the `processes` queue, changes the next process's state to
    /// `Running`, and performs context switch by restoring the next process`s
    /// trap frame into `tf`.
    ///
    /// If there is no process to switch to, returns `None`. Otherwise, returns
    /// `Some` of the next process`s process ID.
    fn switch_to(&mut self, tf: &mut TrapFrame) -> Option<Id> {
        let mut j = self.processes.len();
        for (i, process) in self.processes.iter_mut().enumerate() {
            if process.is_ready() {
                j = i;
                break;
            }
        };

        if j == self.processes.len() {
            None
        } else {
            let mut process = self.processes.remove(j)?;
            process.state = State::Running;
            let id = process.context.tpidr;
            (*tf) = *process.context;
            self.processes.push_front(process);

            Some(id)
        }
    }

    /// Kills currently running process by scheduling out the current process
    /// as `Dead` state. Releases all process resources held by the process,
    /// removes the dead process from the queue, drops the dead process's
    /// instance, and returns the dead process's process ID.
    fn kill(&mut self, tf: &mut TrapFrame) -> Option<Id> {
        self.schedule_out(State::Dead, tf);

        let process = self.processes.pop_back()?;
        let pid = process.context.tpidr;

        if let Some(parent_id) = process.parent {
            if let Some(parent) = self.find_process(parent_id) {
                parent.dead_children.push(process.context.tpidr);
            }
        }

        Some(pid)
    }

    /// Finds a process corresponding with tpidr saved in a trap frame.
    /// Panics if the search fails.
    pub fn find_process(&mut self, id: Id) -> Option<&mut Process> {
        self.processes.iter_mut().find(|process| process.context.tpidr == id)
    }
}

impl fmt::Debug for Scheduler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let len = self.processes.len();
        write!(f, "  [Scheduler] {} processes in the queue\n", len)?;
        for i in 0..len {
            write!(
                f,
                "    queue[{}]: proc({:3})-{:?} \n",
                i, self.processes[i].context.tpidr, self.processes[i].state
            )?;
        }
        Ok(())
    }
}
