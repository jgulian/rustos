use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec;
use core::time::Duration;

use kernel_api::*;
use kernel_api::OsError::BadAddress;
use pi::timer;

use crate::{kprintln, SCHEDULER};
use crate::memory::{PagePermissions, VirtualAddr};
use crate::param::{PAGE_SIZE, USER_IMG_BASE};
use crate::process::{ResourceId, State};
use crate::traps::TrapFrame;

/// Sleep for `ms` milliseconds.
///
/// This system call takes one parameter: the number of milliseconds to sleep.
///
/// In addition to the usual status value, this system call returns one
/// parameter: the approximate true elapsed time from when `sleep` was called to
/// when `sleep` returned.
pub fn sys_sleep(tf: &mut TrapFrame) -> OsResult<()> {
    let ms = tf.xs[0];
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
    })??.into();

    Ok(())
}

fn sys_close(tf: &mut TrapFrame) -> OsResult<()> {
    let id = ResourceId::from(tf.xs[0]);
    SCHEDULER.on_process(tf, |process| {
        process.resources.remove(id)?;
        Ok(())
    })?
}

pub fn sys_read(tf: &mut TrapFrame) -> OsResult<()> {
    let descriptor = tf.xs[0];
    let ptr = tf.xs[1];
    let len = tf.xs[2] as usize;

    let mut buffer = vec![0u8; len];

    let amount_read = SCHEDULER.on_process(tf, |process| {
        process.read(ResourceId::from(descriptor), buffer.as_mut_slice())
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
        process.write(ResourceId::from(descriptor), buffer.as_slice())
    })??;
    tf.xs[0] = amount_written as u64;

    Ok(())
}

fn sys_pipe(tf: &mut TrapFrame) -> OsResult<()> {
    let (ingress, egress) = SCHEDULER.on_process(tf, |process| {
        process.pipe()
    })??;

    tf.xs[0] = ingress.into();
    tf.xs[1] = egress.into();
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
        process.vmap.alloc(VirtualAddr::from(heap_base), PagePermissions::RW, false);
        Ok((heap_base as u64, PAGE_SIZE as u64))
    })??;

    tf.xs[0] = result.0;
    tf.xs[1] = result.1;

    Ok(())
}

fn sys_duplicate(tf: &mut TrapFrame) -> OsResult<()> {
    let descriptor = ResourceId::from(tf.xs[0]);
    let new_descriptor = ResourceId::from(tf.xs[1]);

    SCHEDULER.on_process(tf, |process| {
        process.duplicate(descriptor, new_descriptor)
    })??;

    Ok(())
}

fn sys_seek(_tf: &mut TrapFrame) -> OsResult<()> {
    Err(OsError::Unknown)
}

fn sys_fork(tf: &mut TrapFrame) -> OsResult<()> {
    tf.xs[0] = SCHEDULER.fork(tf).ok_or(OsError::NoVmSpace)?;
    tf.xs[1] = 0;

    Ok(())
}

fn sys_execute(tf: &mut TrapFrame) -> OsResult<()> {
    let mut arguments = vec![0u8; tf.xs[1] as usize];
    let mut environment = vec![0u8; tf.xs[3] as usize];

    copy_from_userspace(tf, tf.xs[0], arguments.as_mut_slice())?;
    copy_from_userspace(tf, tf.xs[2], environment.as_mut_slice())?;

    SCHEDULER.on_process(tf, |process|
        process.execute(arguments.as_slice(), environment.as_slice()))??;
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

fn syscall_to_function(call: Syscall) -> fn(tf: &mut TrapFrame) -> OsResult<()> {
    match call {
        Syscall::Open => sys_open,
        Syscall::Close => sys_close,
        Syscall::Read => sys_read,
        Syscall::Write => sys_write,
        Syscall::Pipe => sys_pipe,
        Syscall::Duplicate => sys_duplicate,
        Syscall::Seek => sys_seek,
        Syscall::Fork => sys_fork,
        Syscall::Execute => sys_execute,
        Syscall::Exit => sys_exit,
        Syscall::Wait => sys_wait,
        Syscall::GetPid => sys_getpid,
        Syscall::Sbrk => sys_sbrk,
        Syscall::Sleep => sys_sleep,
        Syscall::Time => sys_time,
        Syscall::Unknown => |_| Err(OsError::Unknown)
    }
}

pub fn handle_syscall(num: u16, tf: &mut TrapFrame) {
    let call = Syscall::from(num);
    let result = syscall_to_function(call)(tf);
    tf.xs[7] = match result {
        Ok(_) => 1,
        Err(err) => err as u64
    }
}
