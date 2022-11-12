use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec;
use core::time::Duration;

use crate::console::{CONSOLE, kprint};

use crate::process::State;
use crate::traps::TrapFrame;
use crate::{kprintln, Process, SCHEDULER};

use kernel_api::*;
use kernel_api::OsError::BadAddress;
use pi::timer;
use shim::{io, ioerr};
use shim::io::{Read, Write};
use crate::memory::VirtualAddr;
use crate::param::PAGE_SIZE;

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

pub fn sys_open(_tf: &mut TrapFrame) -> OsResult<()> {
    //let file = tf.xs[0];
    //let ptr = tf.xs[1];
    //let len = tf.xs[2] as usize;
    //
    //let mut buffer = vec![0u8; len];
    //
    //if file == 0 {
    //    if CONSOLE.lock().read_exact(buffer.as_mut_slice()) {
    //        tf.xs[7] = OsError::IoErrorTimedOut as u64;
    //    }
    //}
    //copy_into_userspace(tf, ptr, buffer.as_slice());
    unimplemented!("amogus")
}

pub fn sys_read(_tf: &mut TrapFrame) -> OsResult<()> {
    //let file = tf.xs[0];
    //let ptr = tf.xs[1];
    //let len = tf.xs[2] as usize;
    //
    //let mut buffer = vec![0u8; len];
    //
    //if file == 0 {
    //    if CONSOLE.lock().read_exact(buffer.as_mut_slice()) {
    //        tf.xs[7] = OsError::IoErrorTimedOut as u64;
    //    }
    //}
    //copy_into_userspace(tf, ptr, buffer.as_slice());
    unimplemented!("amogus")
}

pub fn sys_write(tf: &mut TrapFrame) -> OsResult<()> {
    let file = tf.xs[0];
    let ptr = tf.xs[1];
    let len = tf.xs[2] as usize;

    let mut buffer = vec![0u8; len];
    copy_from_userspace(tf, ptr, buffer.as_mut_slice())?;

    if file == 0 {
        if CONSOLE.lock().write(buffer.as_slice()).is_err() {
            Err(OsError::IoErrorTimedOut)
        } else {
            Ok(())
        }
    } else {
        Ok(())
    }
}

//TODO: make the functions work across page boundaries
fn copy_from_userspace(tf: &TrapFrame, ptr: u64, buf: &mut [u8]) -> OsResult<()> {
    let virtual_address = VirtualAddr::from(ptr);
    if virtual_address.offset() as usize + buf.len() > PAGE_SIZE {
        return Err(BadAddress);
    }

    SCHEDULER.on_process(tf, |process| -> OsResult<()> {
        let address = process.vmap.translate(VirtualAddr::from(ptr))
            .map_err(|_| BadAddress)?;
        unsafe {
            let real_ptr = address.as_u64() as *const u8;
            buf.copy_from_slice(core::slice::from_raw_parts(real_ptr, buf.len()));
        }

        Ok(())
    })
}

fn copy_into_userspace(tf: &TrapFrame, ptr: u64, buf: &[u8]) -> OsResult<()> {
    let virtual_address = VirtualAddr::from(ptr);
    if virtual_address.offset() as usize + buf.len() > PAGE_SIZE {
        return Err(BadAddress);
    }

    SCHEDULER.on_process(tf, |process| -> OsResult<()> {
        let address = process.vmap.translate(VirtualAddr::from(ptr))
            .map_err(|_| BadAddress)?;
        unsafe {
            let real_ptr = address.as_u64() as *mut u8;
            core::slice::from_raw_parts_mut(real_ptr, buf.len()).copy_from_slice(buf);
        }

        Ok(())
    })
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
            Ok(())
        }
        Syscall::Unknown => {
            Ok(())
        }
    };

    // TODO: this can be simplified with into/from?
    tf.xs[7] = match result {
        Ok(_) => 1,
        Err(err) => err as u64
    }
}
