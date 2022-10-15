use alloc::boxed::Box;
use alloc::collections::vec_deque::VecDeque;
use core::fmt;

use aarch64;
use pi::interrupt::{Controller, Interrupt};
use pi::timer::{tick_in, Timer};

use crate::mutex::Mutex;
use crate::param::{PAGE_MASK, PAGE_SIZE, TICK, USER_IMG_BASE};
use crate::process::{Id, Process, State};
use crate::traps::TrapFrame;
use crate::{IRQ, kprintln, SCHEDULER, shell, VMM};
use crate::console::kprint;

extern "C" {
    fn _start();
}

extern fn run_shell() {
    shell::Shell::new("user0> ").run();
    unsafe { asm!("brk 3" :::: "volatile"); }
    loop { shell::Shell::new("user1> ").run(); }
}

/// Process scheduler for the entire machine.
#[derive(Debug)]
pub struct GlobalScheduler(Mutex<Option<Scheduler>>);

impl GlobalScheduler {
    /// Returns an uninitialized wrapper around a local scheduler.
    pub const fn uninitialized() -> GlobalScheduler {
        GlobalScheduler(Mutex::new(None))
    }

    /// Enter a critical region and execute the provided closure with the
    /// internal scheduler.
    pub fn critical<F, R>(&self, f: F) -> R
        where
            F: FnOnce(&mut Scheduler) -> R,
    {
        let mut guard = self.0.lock();
        f(guard.as_mut().expect("scheduler uninitialized"))
    }


    /// Adds a process to the scheduler's queue and returns that process's ID.
    /// For more details, see the documentation on `Scheduler::add()`.
    pub fn add(&self, process: Process) -> Option<Id> {
        self.critical(move |scheduler| scheduler.add(process))
    }

    /// Performs a context switch using `tf` by setting the state of the current
    /// process to `new_state`, saving `tf` into the current process, and
    /// restoring the next process's trap frame into `tf`. For more details, see
    /// the documentation on `Scheduler::schedule_out()` and `Scheduler::switch_to()`.
    pub fn switch(&self, new_state: State, tf: &mut TrapFrame) -> Id {
        kprintln!("here1");
        self.critical(|scheduler| scheduler.schedule_out(new_state, tf));
        kprintln!("here2");
        self.switch_to(tf)
    }

    pub fn switch_to(&self, tf: &mut TrapFrame) -> Id {
        loop {
            let rtn = self.critical(|scheduler| scheduler.switch_to(tf));
            if let Some(id) = rtn {
                return id;
            }
            kprintln!("looping");
            aarch64::wfe();
        }
    }

    /// Kills currently running process and returns that process's ID.
    /// For more details, see the documentaion on `Scheduler::kill()`.
    #[must_use]
    pub fn kill(&self, tf: &mut TrapFrame) -> Option<Id> {
        self.critical(|scheduler| scheduler.kill(tf))
    }

    /// Starts executing processes in user space using timer interrupt based
    /// preemptive scheduling. This method should not return under normal conditions.
    pub fn start(&self) -> ! {
        IRQ.register(Interrupt::Timer1, Box::new(|tf| {
            kprintln!("tick tick boom");
            tick_in(TICK);
            SCHEDULER.switch(State::Ready, tf);
        }));
        tick_in(TICK);
        Controller::new().enable(Interrupt::Timer1);

        let process = Process::new().expect("unable to create process");

        let top_of_stack = process.stack.top().as_usize();
        let mut trap_frame: TrapFrame = Default::default();
        for x in trap_frame.xs.iter_mut() {
            *x = 0;
        }
        for q in trap_frame.qs.iter_mut() {
            *q = 0_f64;
        }

        trap_frame.sp = _start as *const () as u64;
        trap_frame.tpidr = 0;
        trap_frame.elr = run_shell as *const () as u64;
        trap_frame.spsr = 0;

        self.add(process);

        kprintln!("{}", trap_frame);

        unsafe {
            asm!("mov x0, $0
                  mov sp, x0"
                :: "r"(&trap_frame as *const TrapFrame as u64)
                :: "volatile");
            asm!("bl context_restore");
            asm!("mov x28, 0");
            asm!("mov x29, 0");
        }

        aarch64::eret();

        loop {}
    }

    /// Initializes the scheduler and add userspace processes to the Scheduler
    pub unsafe fn initialize(&self) {
        *self.0.lock() = Some(Scheduler::new());
    }

    // The following method may be useful for testing Phase 3:
    //
    // * A method to load a extern function to the user process's page table.
    //
    // pub fn test_phase_3(&self, proc: &mut Process){
    //     use crate::vm::{VirtualAddr, PagePerm};
    //
    //     let mut page = proc.vmap.alloc(
    //         VirtualAddr::from(USER_IMG_BASE as u64), PagePerm::RWX);
    //
    //     let text = unsafe {
    //         core::slice::from_raw_parts(test_user_process as *const u8, 24)
    //     };
    //
    //     page[0..24].copy_from_slice(text);
    // }
}

#[derive(Debug)]
pub struct Scheduler {
    processes: VecDeque<Process>,
    last_id: Option<Id>,
}

impl Scheduler {
    /// Returns a new `Scheduler` with an empty queue.
    fn new() -> Scheduler {
        Scheduler {
            processes: VecDeque::new(),
            last_id: None
        }
    }

    /// Adds a process to the scheduler's queue and returns that process's ID if
    /// a new process can be scheduled. The process ID is newly allocated for
    /// the process and saved in its `trap_frame`. If no further processes can
    /// be scheduled, returns `None`.
    ///
    /// It is the caller's responsibility to ensure that the first time `switch`
    /// is called, that process is executing on the CPU.
    fn add(&mut self, mut process: Process) -> Option<Id> {
        let new_pid = match self.last_id {
            None => Id::from(0u64),
            Some(pid) => {
                if pid == u64::max_value() {
                    return None;
                }
                Id::from(pid + 1)
            },
        };

        (*process.context).tpidr = new_pid;
        self.processes.push_back(process);

        Some(new_pid)
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
        kprintln!("amogus");
        let mut j = self.processes.len();
        for (i, process) in self.processes.iter_mut().enumerate() {
            kprintln!("proc {} is ready {}", i, process.is_ready());
            if process.is_ready() {
                j = i;
                break;
            }
        };

        let mut process = self.processes.remove(j)?;
        process.state = State::Running;
        let id = process.context.tpidr;
        (*tf) = *process.context;
        self.processes.push_front(process);

        kprintln!("here amogus");

        Some(id)
    }

    /// Kills currently running process by scheduling out the current process
    /// as `Dead` state. Removes the dead process from the queue, drop the
    /// dead process's instance, and returns the dead process's process ID.
    fn kill(&mut self, tf: &mut TrapFrame) -> Option<Id> {
        self.schedule_out(State::Dead, tf);

        let process = self.processes.pop_back()?;
        let pid = process.context.tpidr;
        Some(pid)
    }
}

pub extern "C" fn test_user_process() -> ! {
    loop {
        let ms = 10000;
        let error: u64;
        let elapsed_ms: u64;

        unsafe {
            asm!("mov x0, $2
              svc 1
              mov $0, x0
              mov $1, x7"
                 : "=r"(elapsed_ms), "=r"(error)
                 : "r"(ms)
                 : "x0", "x7"
                 : "volatile");
        }
    }
}

