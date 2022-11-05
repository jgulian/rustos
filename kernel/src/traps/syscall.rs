use alloc::boxed::Box;
use core::time::Duration;

use crate::console::kprint;

use crate::process::State;
use crate::traps::TrapFrame;
use crate::{kprintln, SCHEDULER};

use kernel_api::*;
use pi::timer;

/// Sleep for `ms` milliseconds.
///
/// This system call takes one parameter: the number of milliseconds to sleep.
///
/// In addition to the usual status value, this system call returns one
/// parameter: the approximate true elapsed time from when `sleep` was called to
/// when `sleep` returned.
pub fn sys_sleep(ms: u32, tf: &mut TrapFrame) {
    let started = timer::current_time();
    let sleep_until = started + Duration::from_millis(ms as u64);

    let waiting = State::Waiting(Box::new(move |process| {
        let current_time = timer::current_time();
        let passed = sleep_until < current_time;
        if passed {
            let millis: u64 = (current_time - started).as_millis() as u64;
            kprintln!("{}", millis);
            process.context.xs[0] = millis;
            process.context.xs[8] = 0;
        }
        passed
    }));

    SCHEDULER.switch(waiting, tf);
}

/// Returns current time.
///
/// This system call does not take parameter.
///
/// In addition to the usual status value, this system call returns two
/// parameter:
///  - current time as seconds
///  - fractional part of the current time, in nanoseconds.
pub fn sys_time(tf: &mut TrapFrame) {
    tf.xs[0] = timer::current_time().as_secs();
    tf.xs[1] = timer::current_time().as_nanos() as u64;
}

/// Kills the current process.
///
/// This system call does not take paramer and does not return any value.
pub fn sys_exit(tf: &mut TrapFrame) {
    SCHEDULER.kill(tf).expect("failed to kill process");
    SCHEDULER.switch_to(tf);
}

/// Writes to console.
///
/// This system call takes one parameter: a u8 character to print.
///
/// It only returns the usual status value.
pub fn sys_write(b: u8, _tf: &mut TrapFrame) {
    kprint!("{}", b as char);
}

/// Returns the current process's ID.
///
/// This system call does not take parameter.
///
/// In addition to the usual status value, this system call returns a
/// parameter: the current process's ID.
pub fn sys_getpid(tf: &mut TrapFrame) {
    tf.xs[0] = tf.tpidr;
}

pub fn handle_syscall(num: u16, tf: &mut TrapFrame) {
    //info!("syscall {}", num);

    tf.xs[7] = 1;

    match Syscall::from(num) {
        Syscall::Sleep => {
            sys_sleep(tf.xs[0] as u32, tf);
        }
        Syscall::Time => {
            sys_time(tf);
        }
        Syscall::Exit => {
            sys_exit(tf)
        }
        Syscall::Write => {
            sys_write(tf.xs[0] as u8, tf);
        }
        Syscall::GetPid => {
            sys_getpid(tf);
        }
        Syscall::Sbrk => {}
        Syscall::Unknown => {}
    }
}
