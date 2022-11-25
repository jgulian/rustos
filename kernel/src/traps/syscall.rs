use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec;
use core::time::Duration;

use filesystem::path::Path;
use kernel_api::*;
use kernel_api::OsError::BadAddress;
use pi::timer;
use shim::{io, ioerr};
use shim::io::{Read, Write};

use crate::{kprintln, Process, SCHEDULER};
use crate::console::{CONSOLE, kprint};
use crate::memory::{PagePerm, VirtualAddr};
use crate::param::{PAGE_SIZE, USER_IMG_BASE};
use crate::process::State;
use crate::traps::TrapFrame;

/// Sleep for `ms` milliseconds.
///
/// This system call takes one parameter: the number of milliseconds to sleep.
///
/// In addition to the usual status value, this system call returns one
/// parameter: the approximate true elapsed time from when `sleep` was called to
/// when `sleep` returned.
pub fn sys_sleep(ms: u32, tf: &mut TrapFrame) -> OsResult<()> {
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

    Ok(())
}

/// Returns current time.
///
/// This system call does not take parameter.
///
/// In addition to the usual status value, this system call returns two
/// parameter:
///  - current time as seconds
///  - fractional part of the current time, in nanoseconds.
pub fn sys_time(tf: &mut TrapFrame) -> OsResult<()> {
    tf.xs[0] = timer::current_time().as_secs();
    tf.xs[1] = timer::current_time().as_nanos() as u64;

    Ok(())
}

/// Kills the current process.
///
/// This system call does not take paramer and does not return any value.
pub fn sys_exit(tf: &mut TrapFrame) -> OsResult<()> {
    SCHEDULER.kill(tf).expect("failed to kill process");
    SCHEDULER.switch_to(tf);

    Ok(())
}

pub fn sys_open(tf: &mut TrapFrame) -> OsResult<()> {
    let ptr = tf.xs[0];
    let len = tf.xs[1] as usize;

    let mut buffer = vec![0u8; len];
    copy_from_userspace(tf, ptr, buffer.as_mut_slice())?;
    let path = String::from_utf8_lossy(buffer.as_slice()).to_string();

    tf.xs[0] = SCHEDULER.on_process(tf, |process| {
        process.open(path)
    })??;

    Ok(())
}

pub fn sys_read(tf: &mut TrapFrame) -> OsResult<()> {
    let descriptor = tf.xs[0];
    let ptr = tf.xs[1];
    let len = tf.xs[2] as usize;

    let mut buffer = vec![0u8; len];

    let amount_read = SCHEDULER.on_process(tf, |process| {
        process.read(descriptor, buffer.as_mut_slice())
    })??;

    copy_into_userspace(tf, ptr, &buffer[0..amount_read])?;
    tf.xs[0] = amount_read as u64;

    Ok(())
}

pub fn sys_write(tf: &mut TrapFrame) -> OsResult<()> {
    let descriptor = tf.xs[0];
    let ptr = tf.xs[1];
    let len = tf.xs[2] as usize;

    let mut buffer = vec![0u8; len];
    copy_from_userspace(tf, ptr, buffer.as_mut_slice())?;

    let amount_written = SCHEDULER.on_process(tf, |process| {
        process.write(descriptor, buffer.as_slice())
    })??;
    tf.xs[0] = amount_written as u64;

    Ok(())
}

/// Returns the current process's ID.
///
/// This system call does not take parameter.
///
/// In addition to the usual status value, this system call returns a
/// parameter: the current process's ID.
pub fn sys_getpid(tf: &mut TrapFrame) -> OsResult<()> {
    tf.xs[0] = tf.tpidr;
    Ok(())
}

/// Returns current time.
///
/// This system call does not take parameter.
///
/// In addition to the usual status value, this system call returns two
/// parameter:
///  - current time as seconds
///  - fractional part of the current time, in nanoseconds.
pub fn sys_sbrk(tf: &mut TrapFrame) -> OsResult<()> {
    let result = SCHEDULER.on_process(tf, |process| -> OsResult<(u64, u64)> {
        //TODO: pick a better heap base / allow more sbrks / something might be wrong with is_valid
        let heap_base = USER_IMG_BASE + PAGE_SIZE;
        process.vmap.alloc(VirtualAddr::from(heap_base), PagePerm::RW);
        Ok((heap_base as u64, PAGE_SIZE as u64))
    })??;

    info!("allocated space for process {}", tf.tpidr);
    tf.xs[0] = result.0;
    tf.xs[1] = result.1;

    Ok(())
}

fn sys_fork(tf: &mut TrapFrame) -> OsResult<()> {
    tf.xs[0] = SCHEDULER.fork(tf).ok_or(OsError::NoVmSpace)?;
    Ok(())
}

fn sys_wait(tf: &mut TrapFrame) -> OsResult<()> {
    SCHEDULER.switch(State::Waiting(Box::new(|process| {
        if let Some(id) = process.dead_children.pop() {
            process.context.xs[0] = id;
            true
        } else {
            false
        }
    })), tf);
    Ok(())
}

fn sys_duplicate(tf: &mut TrapFrame) -> OsResult<()> {
    let descriptor = tf.xs[0];
    tf.xs[0] = SCHEDULER.on_process(tf, |process| {
        process.duplicate(descriptor)
    })??;

    Ok(())
}

//TODO: make the functions work across page boundaries
fn copy_from_userspace(_: &TrapFrame, ptr: u64, buf: &mut [u8]) -> OsResult<()> {
    let virtual_address = VirtualAddr::from(ptr);

    if virtual_address.offset() as usize + buf.len() > PAGE_SIZE {
        Err(BadAddress)
    } else {
        buf.copy_from_slice(unsafe { core::slice::from_raw_parts(ptr as *const u8, buf.len()) });
        Ok(())
    }
}

fn copy_into_userspace(_: &TrapFrame, ptr: u64, buf: &[u8]) -> OsResult<()> {
    let virtual_address = VirtualAddr::from(ptr);
    if virtual_address.offset() as usize + buf.len() > PAGE_SIZE {
        return Err(BadAddress);
    }

    if virtual_address.offset() as usize + buf.len() > PAGE_SIZE {
        Err(BadAddress)
    } else {
        let user_slice = unsafe { core::slice::from_raw_parts_mut(ptr as *mut u8, buf.len()) };
        user_slice.copy_from_slice(buf);
        Ok(())
    }
}

pub fn handle_syscall(num: u16, tf: &mut TrapFrame) {
    let result = match Syscall::from(num) {
        Syscall::Sleep => {
            sys_sleep(tf.xs[0] as u32, tf)
        }
        Syscall::Time => {
            sys_time(tf)
        }
        Syscall::Exit => {
            sys_exit(tf)
        }
        Syscall::Open => {
            sys_open(tf)
        }
        Syscall::Read => {
            sys_read(tf)
        }
        Syscall::Write => {
            sys_write(tf)
        }
        Syscall::GetPid => {
            sys_getpid(tf)
        }
        Syscall::Sbrk => {
            info!("sbrk received");
            sys_sbrk(tf)
        }
        Syscall::Fork => {
            info!("elr fork {:x?}", tf.elr);
            sys_fork(tf)
        }
        Syscall::Duplicate => {
            sys_duplicate(tf)
        }
        Syscall::Execute => {
            info!("needs to implement {:?}", Syscall::from(num));
            panic!("called unimplemented syscall");
            Err(OsError::Unknown)
        }
        Syscall::Wait => {
            sys_wait(tf)
        }
        Syscall::Unknown => {
            Err(OsError::Unknown)
        }
    };

    // TODO: this can be simplified with into/from?
    tf.xs[7] = match result {
        Ok(_) => 1,
        Err(err) => {
            err as u64
        }
    };
}
