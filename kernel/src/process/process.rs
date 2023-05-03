use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::borrow::Borrow;
use core::mem;
use core::slice::from_raw_parts;

use aarch64::SPSR_EL1;
use filesystem::filesystem::Filesystem;
use filesystem::path::Path;
use kernel_api::{OsError, OsResult};
use shim::{io, newioerr};
use sync::Mutex;

use crate::{FILESYSTEM, VMM};
use crate::memory::*;
use crate::multiprocessing::spin_lock::SpinLock;
use crate::param::*;
use crate::process::pipe::PipeResource;
use crate::process::resource::{Resource, ResourceId, ResourceList};
use crate::process::State;
use crate::traps::TrapFrame;

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct ProcessId(u64);

impl From<ProcessId> for u64 {
    fn from(val: ProcessId) -> Self {
        val.0
    }
}

impl From<u64> for ProcessId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

/// A structure that represents the complete state of a process.
pub struct Process {
    /// The saved trap frame of a process.
    pub context: Box<TrapFrame>,
    /// The page table describing the Virtual Memory of the process
    pub vmap: Arc<SpinLock<UserPageTable>>,
    /// The scheduling state of the process.
    pub state: State,
    /// The resources (files) open by a process
    pub(crate) resources: ResourceList,
    /// Parent process
    pub(crate) parent: Option<ProcessId>,
    /// Last dead process
    pub(crate) dead_children: Vec<ProcessId>,
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
        Ok(Process {
            context: Box::default(),
            vmap: Arc::new(SpinLock::new(UserPageTable::new())),
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
        let mut p = Process::do_load(pn)?;

        p.context.sp = Process::get_stack_top().as_u64();
        p.context.elr = Process::get_image_base().as_u64();
        p.context.ttbr0 = VMM.get_baddr().as_u64();
        p.context.ttbr1 = p.vmap.lock(|vmap| vmap.get_baddr().as_u64()).unwrap();
        p.context.spsr = SPSR_EL1::F | SPSR_EL1::A | SPSR_EL1::D;

        Ok(p)
    }

    /// Creates a process and open a file with given path.
    /// Allocates one page for stack with read/write permission, and N pages with read/write/execute
    /// permission to load file's contents.
    fn do_load(pn: &Path) -> OsResult<Process> {
        let process = Process::new()?;
        let user_image = process.vmap.lock(|vmap| {
            vmap.new_stack(Process::get_stack_base());
            vmap.alloc(Process::get_image_base(), PagePermissions::RWX)
        }).unwrap();

        let mut file = FILESYSTEM
            .borrow()
            .open(pn)
            .map_err(|_| OsError::IoError)?
            .into_file()
            .map_err(|_| OsError::NoEntry)?;

        let _amount_read = file.read(user_image).map_err(|_| OsError::IoError)?;
        Ok(process)
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
        VirtualAddr::from(u64::MAX)
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
    pub fn can_run(&mut self) -> bool {
        if let State::Waiting(p) = &mut self.state {
            let mut poll = mem::replace(p, Box::new(|_| false));
            if poll(self) {
                self.state = State::Ready;
            } else if let State::Waiting(pr) = &mut self.state {
                let _ = mem::replace(pr, poll);
            }
        }

        match self.state {
            State::Ready => true,
            _ => false,
        }
    }

    pub fn id(&self) -> ProcessId {
        ProcessId(self.context.tpidr)
    }

    //TODO: limit number of open files
    pub fn open(&mut self, path_name: String) -> OsResult<ResourceId> {
        let path = Path::try_from(path_name.as_str())?;
        let file = FILESYSTEM
            .borrow()
            .open(&path)?
            .into_file()
            .map_err(|_| newioerr!(NotFound))?;
        Ok(self.resources.insert(Resource::File(file)))
    }

    pub fn close(&mut self, id: ResourceId) -> OsResult<()> {
        self.resources.remove(id)
    }

    //TODO: fix seek/clean and make write have the same semantics
    pub fn read(&mut self, id: ResourceId, buffer: &mut [u8]) -> OsResult<usize> {
        match self.resources.get(id)? {
            Resource::File(ref mut file) => match file.read(buffer) {
                Ok(value) => Ok(value),
                Err(err) => Err(err.into()),
            },
        }
    }

    pub fn write(&mut self, id: ResourceId, buffer: &[u8]) -> OsResult<usize> {
        match self.resources.get(id)? {
            Resource::File(ref mut file) => match file.write(buffer) {
                Ok(value) => Ok(value),
                Err(err) => Err(err.into()),
            },
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
            Resource::File(file) => Resource::File(file.duplicate()?),
        };
        self.resources.insert_with_id(new_id, duplicate)
    }

    pub fn fork(&mut self) -> OsResult<Process> {
        let mut new_process = Process {
            context: Box::new(*self.context),
            vmap: Arc::new(SpinLock::new(UserPageTable::new())),
            state: State::Ready,
            resources: self.resources.duplicate()?,
            parent: Some(ProcessId::from(self.context.tpidr)),
            dead_children: Vec::new(),
            current_directory: self.current_directory.clone(),
        };

        new_process.context.xs[0] = 0;
        new_process.context.xs[1] = 1;
        new_process.context.xs[7] = OsError::Ok as u64;
        new_process.context.ttbr0 = VMM.get_baddr().as_u64();
        new_process.vmap.lock(|new_vmap| -> OsResult<()> {
            self.vmap.lock(|old_vmap| -> OsResult<()>  {
                if cfg!(feature = "cow_fork") {
                    old_vmap
                        .allocated()
                        .try_for_each(|(virtual_address, l3_entry)| {
                            new_vmap.cow(virtual_address, l3_entry)
                        })?;
                } else {
                    for (virtual_address, l3_entry) in old_vmap.allocated() {
                        let page = new_vmap
                            .alloc(virtual_address, l3_entry.permissions());
                        let source = unsafe { from_raw_parts(l3_entry.address() as *const u8, PAGE_SIZE) };
                        page.copy_from_slice(source);
                    }
                }
                Ok(())
            }).unwrap()?;
            Ok(new_process.context.ttbr1 = new_vmap.get_baddr().as_u64())
        }).unwrap()?;

        Ok(new_process)
    }

    pub fn execute(&mut self, arguments: &[u8], environment: &[u8]) -> OsResult<()> {
        let argument_vec = parse_execute(arguments);
        let _environment_vec = parse_execute(environment);

        let mut absolute_path = Path::root();
        absolute_path.join_str(
            argument_vec
                .first()
                .ok_or(newioerr!(InvalidFilename))?
                .as_str(),
        )?;

        let mut program_file = FILESYSTEM
            .borrow()
            .open(&absolute_path)?
            .into_file()
            .map_err(|_| newioerr!(InvalidFilename))?;

        self.vmap = Arc::new(SpinLock::new(UserPageTable::new()));

        let (_, stack) = self.vmap.lock(|vmap| vmap.new_stack(Process::get_stack_base())).unwrap();
        let mut stack_data = Vec::new();

        // TODO: clean this; actually this is just broken
        stack_data.extend_from_slice(&(arguments.len() as u64).to_be_bytes());
        stack_data.extend_from_slice(&(environment.len() as u64).to_be_bytes());
        arguments
            .split(|x| *x == 0)
            .for_each(|arg| stack_data.extend(arg.iter().rev().chain(&[0])));
        environment
            .split(|x| *x == 0)
            .for_each(|arg| stack_data.extend(arg.iter().rev().chain(&[0])));

        let stack_size = stack.len();
        stack_data.reverse();
        stack[stack_size - stack_data.len()..].copy_from_slice(stack_data.as_slice());

        let user_image = self
            .vmap.lock(|vmap| vmap.alloc(Process::get_image_base(), PagePermissions::RWX)).unwrap();

        program_file.read(user_image)?;

        self.context.sp = Process::get_stack_top().as_u64() - (stack_data.len() as u64);
        self.context.elr = Process::get_image_base().as_u64();
        self.context.ttbr0 = VMM.get_baddr().as_u64();
        self.context.ttbr1 = self.vmap.lock(|vmap| vmap.get_baddr().as_u64()).unwrap();
        self.context.spsr = SPSR_EL1::F | SPSR_EL1::A | SPSR_EL1::D;

        Ok(())
    }

    pub fn clone(&mut self, start_address: u64, argument: u64) -> OsResult<Process> {
        let mut new_process = Process {
            context: Box::new(*self.context),
            vmap: self.vmap.clone(),
            state: State::Ready,
            resources: self.resources.duplicate()?,
            parent: Some(ProcessId::from(self.context.tpidr)),
            dead_children: Vec::new(),
            current_directory: self.current_directory.clone(),
        };

        let (stack_address, _) = self.vmap.lock(|vmap| {
            vmap.new_stack(Process::get_stack_base())
        }).unwrap();

        new_process.context.xs[0] = argument;
        new_process.context.xs[7] = OsError::Ok as u64;
        new_process.context.sp = stack_address.as_u64() + PAGE_SIZE as u64 - 1;
        new_process.context.elr = start_address;

        Ok(new_process)
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
