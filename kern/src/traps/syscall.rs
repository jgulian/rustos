use alloc::boxed::Box;
use alloc::string::ToString;
use core::time::Duration;

use crate::console::{CONSOLE, kprint};
use crate::process::State;
use crate::traps::TrapFrame;
use crate::{kprintln, SCHEDULER};
use kernel_api::*;
use pi::timer;
use pi::timer::Timer;

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

/// Kills current process.
///
/// This system call does not take paramer and does not return any value.
pub fn sys_exit(tf: &mut TrapFrame) {
    kprintln!("Any killers?");
    SCHEDULER.kill(tf).expect("failed to kill process");
    kprintln!("AMOGUS");
    SCHEDULER.switch_to(tf);
}

/// Write to console.
///
/// This system call takes one parameter: a u8 character to print.
///
/// It only returns the usual status value.
pub fn sys_write(b: u8, tf: &mut TrapFrame) {
    kprint!("{}", b as char);
}

/// Returns current process's ID.
///
/// This system call does not take parameter.
///
/// In addition to the usual status value, this system call returns a
/// parameter: the current process's ID.
pub fn sys_getpid(tf: &mut TrapFrame) {
    tf.xs[0] = tf.tpidr;
}

pub fn handle_syscall(num: u16, tf: &mut TrapFrame) {
    match num as usize {
        NR_SLEEP => {
            let time: u32 = tf.xs[0] as u32;
            sys_sleep(time, tf);
        }
        NR_TIME => {
            sys_time(tf);
        }
        NR_WRITE => {
            let b: u8 = tf.xs[0] as u8;
            sys_write(b, tf);
        }
        NR_EXIT => {
            sys_exit(tf);
        }
        NR_GETPID => {
            sys_getpid(tf);
        }
        _ => {}
    }
}
