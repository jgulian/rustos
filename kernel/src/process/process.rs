use alloc::boxed::Box;
use alloc::vec::Vec;
use core::borrow::Borrow;
use core::mem;

use aarch64;
use aarch64::SPSR_EL1;
use filesystem::fs2::FileSystem2;
use filesystem::path::Path;
use kernel_api::{OsError, OsResult};
use shim::io;

use crate::FILESYSTEM;
use crate::memory::*;
use crate::param::*;
use crate::process::{Stack, State};
use crate::process::resource::Resource;
use crate::traps::TrapFrame;

/// Type alias for the type of a process ID.
pub type Id = u64;

/// A structure that represents the complete state of a process.
#[derive(Debug)]
pub struct Process {
    /// The saved trap frame of a process.
    pub context: Box<TrapFrame>,
    /// The memory allocation used for the process's stack.
    /// TODO: remove this its not useful
    pub stack: Stack,
    /// The page table describing the Virtual Memory of the process
    pub vmap: Box<UserPageTable>,
    /// The scheduling state of the process.
    pub state: State,
    /// The resources (files) open by a process
    pub(crate) resources: Vec<Resource>,
}

impl Process {
    /// Creates a new process with a zeroed `TrapFrame` (the default), a zeroed
    /// stack of the default size, and a state of `Ready`.
    ///
    /// If enough memory could not be allocated to start the process, returns
    /// `None`. Otherwise returns `Some` of the new `Process`.
    pub fn new() -> OsResult<Process> {
        let stack = Stack::new().ok_or(OsError::NoMemory)?;
        Ok(Process {
            context: Box::new(Default::default()),
            stack,
            vmap: Box::new(UserPageTable::new()),
            state: State::Ready,
            resources: Vec::new(),
        })
    }

    /// Loads a program stored in the given path by calling `do_load()` method.
    /// Sets trapframe `context` corresponding to its page table.
    /// `sp` - the address of stack top
    /// `elr` - the address of image base.
    /// `ttbr0` - the base address of kernel2 page table
    /// `ttbr1` - the base address of user page table
    /// `spsr` - `F`, `A`, `D` bit should be set.
    ///
    /// Returns Os Error if do_load fails.
    pub fn load(pn: &Path) -> OsResult<Process> {
        use crate::VMM;

        let mut p = Process::do_load(pn)?;

        p.context.sp = Process::get_stack_top().as_u64();
        p.context.elr = Process::get_image_base().as_u64();
        p.context.ttbr0 = VMM.get_baddr().as_u64();
        p.context.ttbr1 = p.vmap.get_baddr().as_u64();
        p.context.spsr = SPSR_EL1::F | SPSR_EL1::A | SPSR_EL1::D;

        Ok(p)
    }

    /// Creates a process and open a file with given path.
    /// Allocates one page for stack with read/write permission, and N pages with read/write/execute
    /// permission to load file's contents.
    fn do_load(pn: &Path) -> OsResult<Process> {
        let mut process = Process::new()?;
        process.vmap.alloc(Process::get_stack_base(), PagePerm::RW);
        let user_image = process.vmap.alloc(Process::get_image_base(), PagePerm::RWX);

        let mut file = FILESYSTEM.borrow().open(pn)
            .map_err(|_| OsError::IoError)?
            .into_file().ok_or(OsError::NoEntry)?;

        file.read(user_image).map_err(|e| OsError::IoError)?;
        Ok(process)
    }

    pub fn load_from_kernel(_function: fn()) -> OsResult<Process> {
        unimplemented!("need to fix text");
        //let mut process = Process::new()?;
        //
        //let mut page = process.vmap.alloc(
        //    VirtualAddr::from(USER_IMG_BASE as u64), PagePerm::RWX);
        //
        //let text = unsafe {
        //    core::slice::from_raw_parts(function as *const u8, 24)
        //};
        //
        //page[0..24].copy_from_slice(text);
        //Err(OsError::NoMemory)
    }

    /// Returns the highest `VirtualAddr` that is supported by this system.
    pub fn get_max_va() -> VirtualAddr {
        VirtualAddr::from(u64::max_value())
    }

    /// Returns the `VirtualAddr` represents the base address of the user
    /// memory space.
    pub fn get_image_base() -> VirtualAddr {
        VirtualAddr::from(USER_IMG_BASE)
    }

    /// Returns the `VirtualAddr` represents the base address of the user
    /// process's stack.
    pub fn get_stack_base() -> VirtualAddr {
        VirtualAddr::from(u64::max_value() & PAGE_MASK as u64)
    }

    /// Returns the `VirtualAddr` represents the top of the user process's
    /// stack.
    pub fn get_stack_top() -> VirtualAddr {
        VirtualAddr::from(u64::max_value())
    }

    /// Returns `true` if this process is ready to be scheduled.
    ///
    /// This functions returns `true` only if one of the following holds:
    ///
    ///   * The state is currently `Ready`.
    ///
    ///   * An event being waited for has arrived.
    ///
    ///     If the process is currently waiting, the corresponding event
    ///     function is polled to determine if the event being waiting for has
    ///     occured. If it has, the state is switched to `Ready` and this
    ///     function returns `true`.
    ///
    /// Returns `false` in all other cases.
    pub fn is_ready(&mut self) -> bool {
        //TODO: clean up, also go through code base and replace a lot of match statements with these
        // lol
        if let State::Waiting(p) = &mut self.state {
            let mut poll = mem::replace(p, Box::new(|_| false));
            if poll(self) {
                self.state = State::Ready;
            } else {
                if let State::Waiting(pr) = &mut self.state {
                    let _ = mem::replace(pr, poll);
                }
            }
        }

        match self.state {
            State::Ready => true,
            _ => false,
        }
    }
}
