use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::borrow::{Borrow};
use core::mem;
use core::mem::zeroed;

use core::ptr::write_volatile;
use core::slice::from_raw_parts;

use aarch64;
use aarch64::SPSR_EL1;
use filesystem::fs2::FileSystem2;
use filesystem::path::Path;
use kernel_api::{OsError, OsResult};
use shim::{io, newioerr};
use shim::io::{Write};

use crate::{FILESYSTEM, VMM};
use crate::console::kprintln;
use crate::memory::*;
use crate::param::*;
use crate::process::{Stack, State};
use crate::process::pipe::PipeResource;
use crate::process::resource::{Resource, ResourceId, ResourceList};
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
    pub(crate) resources: ResourceList,
    /// Parent process
    pub(crate) parent: Option<Id>,
    /// Last dead process
    pub(crate) dead_children: Vec<Id>,
    /// Current Working Directory
    current_directory: Path,
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
            resources: ResourceList::new(),
            parent: None,
            dead_children: Vec::new(),
            current_directory: Path::root(),
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
        process.vmap.alloc(Process::get_stack_base(), PagePermissions::RW);
        let user_image = process.vmap.alloc(Process::get_image_base(), PagePermissions::RWX);

        let mut file = FILESYSTEM.borrow().open(pn)
            .map_err(|_| OsError::IoError)?
            .into_file().ok_or(OsError::NoEntry)?;

        file.read(user_image).map_err(|_e| OsError::IoError)?;
        Ok(process)
    }

    /// Returns the highest `VirtualAddr` that is supported by this system.
    pub fn get_max_va() -> VirtualAddr {
        VirtualAddr::from(u64::MAX)
    }

    /// Returns the `VirtualAddr` represents the base address of the user
    /// memory space.
    pub fn get_image_base() -> VirtualAddr {
        VirtualAddr::from(USER_IMG_BASE)
    }

    /// Returns the `VirtualAddr` represents the base address of the user
    /// process's stack.
    pub fn get_stack_base() -> VirtualAddr {
        VirtualAddr::from(u64::MAX & PAGE_MASK as u64)
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

    //TODO: limit number of open files
    pub fn open(&mut self, path_name: String) -> OsResult<ResourceId> {
        let path = Path::try_from(path_name)?;
        let file = FILESYSTEM.borrow()
            .open(&path)?
            .into_file().ok_or(newioerr!(NotFound))?;
        Ok(self.resources.insert(Resource::File(file)))
    }

    pub fn close(&mut self, id: ResourceId) -> OsResult<()> {
        self.resources.remove(id)
    }

    //TODO: fix seek/clean and make write have the same semantics
    pub fn read(&mut self, id: ResourceId, buffer: &mut [u8]) -> OsResult<usize> {
        match self.resources.get(id)? {
            Resource::File(ref mut file) => {
                match file.read(buffer) {
                    Ok(value) => Ok(value),
                    Err(err) => Err(err.into())
                }
            }
        }
    }

    pub fn write(&mut self, id: ResourceId, buffer: &[u8]) -> OsResult<usize> {
        match self.resources.get(id)? {
            Resource::File(ref mut file) => {
                match file.write(buffer) {
                    Ok(value) => Ok(value),
                    Err(err) => Err(err.into())
                }
            }
        }
    }

    pub fn pipe(&mut self) -> OsResult<(ResourceId, ResourceId)> {
        let (writer, reader) = PipeResource::new_pair();
        let writer_id = self.resources.insert(Resource::File(Box::new(writer)));
        let reader_id = self.resources.insert(Resource::File(Box::new(reader)));
        Ok((writer_id, reader_id))
    }

    pub fn duplicate(&mut self, id: ResourceId, new_id: ResourceId) -> OsResult<()> {
        let resource = self.resources.get(id)?;
        let duplicate = match resource {
            Resource::File(file) => {
                Resource::File(file.duplicate()?)
            }
        };
        self.resources.insert_with_id(new_id, duplicate)
    }

    pub fn fork(&mut self, id: Id) -> OsResult<Process> {
        let stack = Stack::new().ok_or(newioerr!(OutOfMemory))?;
        let mut new_process = Process {
            context: Box::new(*self.context),
            stack,
            vmap: Box::new(UserPageTable::new()),
            state: State::Ready,
            resources: self.resources.duplicate()?,
            parent: Some(self.context.tpidr),
            dead_children: Vec::new(),
            current_directory: self.current_directory.clone(),
        };

        new_process.context.xs[0] = 0;
        new_process.context.xs[1] = 1;
        new_process.context.xs[7] = OsError::Ok as u64;
        new_process.context.ttbr0 = VMM.get_baddr().as_u64();
        new_process.context.ttbr1 = new_process.vmap.get_baddr().as_u64();
        new_process.context.tpidr = id;

        if cfg!(feature = "cow_fork") {
            self.vmap.allocated().try_for_each(|(virtual_address, l3_entry)| {
                new_process.vmap.cow(virtual_address, l3_entry)
            })?;
        } else {
            for (virtual_address, l3_entry) in self.vmap.allocated() {
                let page = new_process.vmap.alloc(virtual_address, l3_entry.permissions());
                let source = unsafe { from_raw_parts(l3_entry.address() as *const u8, PAGE_SIZE) };
                page.copy_from_slice(source);
            }
        }


        Ok(new_process)
    }

    pub fn execute(&mut self, arguments: &[u8], environment: &[u8]) -> OsResult<()> {
        let argument_vec = parse_execute(arguments);
        let _environment_vec = parse_execute(environment);

        let path = Path::try_from(argument_vec.first()
            .ok_or(newioerr!(InvalidFilename))?
            .clone())?;
        let mut absolute_path = Path::root();
        absolute_path.append(&path);

        let mut program_file = FILESYSTEM.borrow().open(&absolute_path)?
            .into_file().ok_or(newioerr!(InvalidFilename))?;

        info!("Execute");

        self.vmap = Box::new(UserPageTable::new());

        let stack = self.vmap.alloc(Process::get_stack_base(), PagePermissions::RW);
        let mut stack_data = Vec::new();

        // TODO: clean this; actually this is just broken
        stack_data.extend_from_slice(&(arguments.len() as u64).to_be_bytes());
        stack_data.extend_from_slice(&(environment.len() as u64).to_be_bytes());
        arguments.split(|x| *x == 0).for_each(|arg|
            stack_data.extend(arg.iter().rev().chain(&[0])));
        environment.split(|x| *x == 0).for_each(|arg|
            stack_data.extend(arg.iter().rev().chain(&[0])));

        let stack_size = stack.len();
        stack_data.reverse();
        stack[stack_size - stack_data.len()..].copy_from_slice(stack_data.as_slice());

        let user_image = self.vmap.alloc(Process::get_image_base(), PagePermissions::RW);
        program_file.read(user_image)?;

        self.context.sp = Process::get_stack_top().as_u64() - (stack_data.len() as u64);
        self.context.elr = Process::get_image_base().as_u64();
        self.context.ttbr0 = VMM.get_baddr().as_u64();
        self.context.ttbr1 = self.vmap.get_baddr().as_u64();
        self.context.spsr = SPSR_EL1::F | SPSR_EL1::A | SPSR_EL1::D;

        Ok(())
    }
}

fn parse_execute(data: &[u8]) -> Vec<String> {
    let mut result = Vec::new();
    let mut last_start = 0;
    for (i, c) in data.iter().enumerate() {
        if *c == 0 {
            result.push(String::from_utf8_lossy(&data[last_start..i]).to_string());
            last_start = i + 1;
        }
    }

    if last_start != data.len() {
        result.push(String::from_utf8_lossy(&data[last_start..]).to_string());
    }

    result
}

unsafe fn zero_page(va: VirtualAddr) {
    let mut iter: *mut u64 = va.as_ptr() as *mut u64;
    let end: *mut u64 = iter.add(PAGE_ALIGN / core::mem::size_of::<u64>() - 1);

    while iter < end {
        write_volatile(iter, zeroed());
        iter = iter.add(1);
    }
}