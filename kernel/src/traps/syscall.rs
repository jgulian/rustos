use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec;
use core::ops::Add;
use core::time::Duration;

use kernel_api::*;
use kernel_api::OsError::BadAddress;
use pi::timer;
use sync::Mutex;

use crate::memory::{PagePermissions, VirtualAddr};
use crate::param::{PAGE_SIZE, USER_IMG_BASE};
use crate::process::{ResourceId, State};
use crate::SCHEDULER;
use crate::scheduling::SwitchTrigger;
use crate::traps::TrapFrame;

/// Sleep for `ms` milliseconds.
///
/// This system call takes one parameter: the number of milliseconds to sleep.
///
/// In addition to the usual status value, this system call returns one
/// parameter: the approximate true elapsed time from when `sleep` was called to
/// when `sleep` returned.
fn sys_sleep(trap_frame: &mut TrapFrame) -> OsResult<()> {
    let ms = trap_frame.xs[0];
    let started = timer::current_time();
    let sleep_until = started + Duration::from_millis(ms);

    let waiting = State::Waiting(Box::new(move |process| {
        let current_time = timer::current_time();
        let passed = sleep_until < current_time;
        //info!(
        //    "checking {:?} < {:?}, {}",
        //    sleep_until, current_time, passed
        //);
        if passed {
            let millis: u64 = (current_time - started).as_millis() as u64;
            process.context.xs[0] = millis;
            process.context.xs[8] = 0;
        }
        passed
    }));

    SCHEDULER.switch(trap_frame, SwitchTrigger::Force, waiting)?;

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
fn sys_time(tf: &mut TrapFrame) -> OsResult<()> {
    tf.xs[0] = timer::current_time().as_secs();
    tf.xs[1] = timer::current_time().as_nanos() as u64;

    Ok(())
}

/// Kills the current process.
///
/// This system call does not take paramer and does not return any value.
fn sys_exit(trap_frame: &mut TrapFrame) -> OsResult<()> {
    SCHEDULER.switch(trap_frame, SwitchTrigger::Force, State::Dead)?;
    Ok(())
}

fn sys_open(tf: &mut TrapFrame) -> OsResult<()> {
    let ptr = tf.xs[0];
    let len = tf.xs[1] as usize;

    let mut buffer = vec![0u8; len];
    copy_from_userspace(tf, ptr, buffer.as_mut_slice())?;
    let path = String::from_utf8_lossy(buffer.as_slice()).to_string();

    tf.xs[0] = SCHEDULER
        .on_process(tf, |process| process.open(path))??
        .into();

    Ok(())
}

fn sys_close(tf: &mut TrapFrame) -> OsResult<()> {
    let id = ResourceId::from(tf.xs[0]);
    SCHEDULER.on_process(tf, |process| {
        process.resources.remove(id)?;
        Ok(())
    })?
}

fn sys_read(tf: &mut TrapFrame) -> OsResult<()> {
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

fn sys_write(tf: &mut TrapFrame) -> OsResult<()> {
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
    let (ingress, egress) = SCHEDULER.on_process(tf, |process| process.pipe())??;

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
fn sys_getpid(tf: &mut TrapFrame) -> OsResult<()> {
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
fn sys_sbrk(tf: &mut TrapFrame) -> OsResult<()> {
    let result = SCHEDULER.on_process(tf, |process| -> OsResult<(u64, u64)> {
        //TODO: pick a better heap base / allow more sbrks / something might be wrong with is_valid
        let mut heap_base = USER_IMG_BASE + PAGE_SIZE;

        process.vmap.lock(|vmap| {
            while vmap.is_valid(VirtualAddr::from(heap_base - USER_IMG_BASE)) {
                heap_base += PAGE_SIZE;
            }
            vmap.alloc(VirtualAddr::from(heap_base), PagePermissions::RW);
        }).unwrap();

        Ok((heap_base as u64, PAGE_SIZE as u64))
    })??;

    tf.xs[0] = result.0;
    tf.xs[1] = result.1;

    Ok(())
}

fn sys_duplicate(tf: &mut TrapFrame) -> OsResult<()> {
    let descriptor = ResourceId::from(tf.xs[0]);
    let new_descriptor = ResourceId::from(tf.xs[1]);

    SCHEDULER.on_process(tf, |process| process.duplicate(descriptor, new_descriptor))??;

    Ok(())
}

fn sys_seek(_tf: &mut TrapFrame) -> OsResult<()> {
    Err(OsError::Unknown)
}

fn sys_fork(trap_frame: &mut TrapFrame) -> OsResult<()> {
    let process = SCHEDULER.on_process(trap_frame, |process| process.fork())??;
    let process_id = SCHEDULER.add(process)?;

    trap_frame.xs[0] = process_id.into();
    trap_frame.xs[1] = 0;

    Ok(())
}

fn sys_execute(tf: &mut TrapFrame) -> OsResult<()> {
    let mut arguments = vec![0u8; tf.xs[1] as usize];
    let mut environment = vec![0u8; tf.xs[3] as usize];

    copy_from_userspace(tf, tf.xs[0], arguments.as_mut_slice())?;
    copy_from_userspace(tf, tf.xs[2], environment.as_mut_slice())?;

    SCHEDULER.on_process(tf, |process| {
        process.execute(arguments.as_slice(), environment.as_slice())
    })??;
    Ok(())
}

fn sys_wait(trap_frame: &mut TrapFrame) -> OsResult<()> {
    let use_timeout = trap_frame.xs[1] == 1;
    let timeout_end = pi::timer::current_time() + Duration::from_millis(trap_frame.xs[2]);

    SCHEDULER.switch(
        trap_frame,
        SwitchTrigger::Force,
        State::Waiting(Box::new(move |process| {
            if let Some(id) = process.dead_children.pop() {
                process.context.xs[0] = id.into();
                process.context.xs[1] = 0;
                true
            } else if use_timeout && timeout_end < timer::current_time() {
                process.context.xs[0] = 0;
                process.context.xs[1] = 1;
                true
            } else {
                false
            }
        })),
    )?;

    Ok(())
}

fn clone(trap_frame: &mut TrapFrame) -> OsResult<()> {
    let function_address = trap_frame.xs[0];
    let data = trap_frame.xs[1];
    let process = SCHEDULER.on_process(trap_frame, |process| process.clone(function_address, data))??;
    let process_id = SCHEDULER.add(process)?;

    trap_frame.xs[0] = process_id.into();

    Ok(())
}

//TODO: make the functions work across page boundaries
//TODO: this is fundamentally unsafe
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
        Syscall::Clone => clone,
        Syscall::Sbrk => sys_sbrk,
        Syscall::Sleep => sys_sleep,
        Syscall::Time => sys_time,
        Syscall::Unknown => |_| Err(OsError::Unknown),
    }
}

pub fn handle_syscall(num: u16, trap_frame: &mut TrapFrame) {
    let call = Syscall::from(num);
    let result = syscall_to_function(call)(trap_frame);
    trap_frame.xs[7] = match result {
        Ok(_) => 1,
        Err(err) => err as u64,
    }
}
