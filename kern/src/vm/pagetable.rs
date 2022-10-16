use core::iter::Chain;
use core::ops::{Deref, DerefMut};
use core::slice::Iter;

use alloc::boxed::Box;
use alloc::fmt;
use core::alloc::{GlobalAlloc, Layout};
use core::fmt::Formatter;
use aarch64::EntryPerm::{KERN_RW, USER_RW};
use aarch64::EntryValid::Valid;

use crate::allocator;
use crate::param::*;
use crate::vm::{PhysicalAddr, VirtualAddr};
use crate::ALLOCATOR;
use fmt::Debug;

use aarch64::vmsa::*;
use shim::const_assert_size;

#[repr(C)]
pub struct Page([u8; PAGE_SIZE]);
const_assert_size!(Page, PAGE_SIZE);

impl Page {
    pub const SIZE: usize = PAGE_SIZE;
    pub const ALIGN: usize = PAGE_SIZE;

    fn layout() -> Layout {
        unsafe { Layout::from_size_align_unchecked(Self::SIZE, Self::ALIGN) }
    }
}

#[repr(C)]
#[repr(align(65536))]
pub struct L2PageTable {
    pub entries: [RawL2Entry; 8192],
}
const_assert_size!(L2PageTable, PAGE_SIZE);

impl L2PageTable {
    /// Returns a new `L2PageTable`
    fn new() -> L2PageTable {
        L2PageTable {
            entries: [RawL2Entry::new(0); 8192],
        }
    }

    /// Returns a `PhysicalAddr` of the pagetable.
    pub fn as_ptr(&self) -> PhysicalAddr {
        PhysicalAddr::from(self.entries.as_ptr() as usize)
    }
}

#[derive(Copy, Clone)]
pub struct L3Entry(RawL3Entry);

impl L3Entry {
    /// Returns a new `L3Entry`.
    fn new() -> L3Entry {
        L3Entry(RawL3Entry::new(0))
    }

    /// Returns `true` if the L3Entry is valid and `false` otherwise.
    fn is_valid(&self) -> bool {
        self.0.get_masked(RawL3Entry::VALID) > 0
    }

    /// Extracts `ADDR` field of the L3Entry and returns as a `PhysicalAddr`
    /// if valid. Otherwise, return `None`.
    fn get_page_addr(&self) -> Option<PhysicalAddr> {
        if self.is_valid() {
            Some(PhysicalAddr::from(self.0.get_masked(RawL3Entry::ADDR)))
        } else {
            None
        }
    }
}

impl From<u64> for L3Entry {
    fn from(v: u64) -> Self {
        L3Entry(RawL3Entry::new(v))
    }
}

#[repr(C)]
#[repr(align(65536))]
pub struct L3PageTable {
    pub entries: [L3Entry; 8192],
}
const_assert_size!(L3PageTable, PAGE_SIZE);

impl L3PageTable {
    /// Returns a new `L3PageTable`.
    fn new() -> L3PageTable {
        L3PageTable {
            entries: [L3Entry::new(); 8192],
        }
    }

    /// Returns a `PhysicalAddr` of the pagetable.
    pub fn as_ptr(&self) -> PhysicalAddr {
        PhysicalAddr::from(self.entries.as_ptr() as usize)
    }
}

#[repr(C)]
#[repr(align(65536))]
pub struct PageTable {
    pub l2: L2PageTable,
    pub l3: [L3PageTable; 2],
}

impl PageTable {
    /// Returns a new `Box` containing `PageTable`.
    /// Entries in L2PageTable should be initialized properly before return.
    fn new(perm: u64) -> Box<PageTable> {
        let mut page_table = Box::new(PageTable{
            l2: L2PageTable::new(),
            l3: [L3PageTable::new(), L3PageTable::new()],
        });

        for i in 0..2 {
            let mut page_entry = RawL2Entry::new(0);
            page_entry.set_value(perm, RawL2Entry::AP);
            page_entry.set_value((*page_table).l3[i].as_ptr().as_u64(), RawL2Entry::ADDR);

            (*page_table).l2.entries[i] = page_entry;
        }

        page_table
    }

    /// Returns the (L2index, L3index) extracted from the given virtual address.
    /// Since we are only supporting 1GB virtual memory in this system, L2index
    /// should be smaller than 2.
    ///
    /// # Panics
    ///
    /// Panics if the virtual address is not properly aligned to page size.
    /// Panics if extracted L2index exceeds the number of L3PageTable.
    fn locate(va: VirtualAddr) -> (usize, usize) {
        let l2_index = va.level2_index();
        let l3_index = va.level3_index();
        if 2 <= l2_index {
            panic!("l2 index out of bounds");
        }

        (l2_index as usize, l3_index as usize)
    }

    /// Returns `true` if the L3entry indicated by the given virtual address is valid.
    /// Otherwise, `false` is returned.
    pub fn is_valid(&self, va: VirtualAddr) -> bool {
        self.get_entry(va).is_valid()
    }

    /// Returns `true` if the L3entry indicated by the given virtual address is invalid.
    /// Otherwise, `true` is returned.
    pub fn is_invalid(&self, va: VirtualAddr) -> bool {
        !self.is_valid(va)
    }

    /// Set the given RawL3Entry `entry` to the L3Entry indicated by the given virtual
    /// address.
    pub fn set_entry(&mut self, va: VirtualAddr, entry: RawL3Entry) -> &mut Self {
        *self.get_entry_mut(va) = L3Entry(entry);
        self
    }

    /// Returns a base address of the pagetable. The returned `PhysicalAddr` value
    /// will point the start address of the L2PageTable.
    pub fn get_baddr(&self) -> PhysicalAddr {
        self.l2.as_ptr()
    }

    fn get_entry(&self, va: VirtualAddr) -> &L3Entry {
        let (l2_index, l3_index) = PageTable::locate(va);
        let l2_entry = self.l2.entries[l2_index];
        let l3_table = PhysicalAddr::from(l2_entry.get_masked(RawL2Entry::ADDR) as usize);
        if self.l3[0].as_ptr() == l3_table {
            &self.l3[0].entries[l3_index]
        } else {
            &self.l3[1].entries[l3_index]
        }
    }

    fn get_entry_mut(&mut self, va: VirtualAddr) -> &mut L3Entry {
        let (l2_index, l3_index) = PageTable::locate(va);
        let l2_entry = self.l2.entries[l2_index];
        let l3_table = PhysicalAddr::from(l2_entry.get_masked(RawL2Entry::ADDR) as usize);
        if self.l3[0].as_ptr() == l3_table {
            &mut self.l3[0].entries[l3_index]
        } else {
            &mut self.l3[1].entries[l3_index]
        }
    }
}

impl<'a> IntoIterator for &'a PageTable {
    type Item = &'a L3Entry;
    type IntoIter = Chain<Iter<'a, L3Entry>, Iter<'a, L3Entry>>;

    fn into_iter(self) -> Self::IntoIter {
        self.l3[0].entries.iter().chain(self.l3[1].entries.iter())
    }
}

// FIXME: Implement `IntoIterator` for `&PageTable`.

pub struct KernPageTable(Box<PageTable>);

impl KernPageTable {
    /// Returns a new `KernPageTable`. `KernPageTable` should have a `Pagetable`
    /// created with `KERN_RW` permission.
    ///
    /// Set L3entry of ARM physical address starting at 0x00000000 for RAM and
    /// physical address range from `IO_BASE` to `IO_BASE_END` for peripherals.
    /// Each L3 entry should have correct value for lower attributes[10:0] as well
    /// as address[47:16]. Refer to the definition of `RawL3Entry` in `vmsa.rs` for
    /// more details.
    pub fn new() -> KernPageTable {
        let mut page_table = PageTable::new(KERN_RW);
        let (start, end) = allocator::memory_map().unwrap_or((0, 0));
        let page_start = allocator::util::align_up(start, PAGE_ALIGN);
        let page_end = allocator::util::align_down(end, PAGE_ALIGN);
        let page_count = (page_end - page_start) / PAGE_SIZE;

        for i in 0..page_count {
            let address = page_start + PAGE_SIZE * i;
            let entry = (*page_table).get_entry_mut(VirtualAddr::from(address));
            entry.0.set_value(address as u64, RawL3Entry::ADDR);
            entry.0.set_value(EntrySh::ISh as u64, RawL3Entry::SH);
            entry.0.set_value(KERN_RW as u64, RawL3Entry::AP);
            entry.0.set_value(EntryAttr::Mem, RawL3Entry::ATTR);
            entry.0.set_value(0b1_u64, RawL3Entry::TYPE);
            entry.0.set_value(0b1_u64, RawL3Entry::VALID);
        }

        let device_start = allocator::util::align_down(IO_BASE, PAGE_ALIGN);
        let device_end = allocator::util::align_up(IO_BASE_END, PAGE_ALIGN);
        let device_page_count = (device_end - device_start) / PAGE_SIZE;

        for i in 0..device_page_count {
            let address = device_start + PAGE_SIZE * i;
            let entry = (*page_table).get_entry_mut(VirtualAddr::from(address));
            entry.0.set_value(address as u64, RawL3Entry::ADDR);
            entry.0.set_value(EntrySh::ISh as u64, RawL3Entry::SH);
            entry.0.set_value(KERN_RW as u64, RawL3Entry::AP);
            entry.0.set_value(EntryAttr::Dev, RawL3Entry::ATTR);
            entry.0.set_value(0b1_u64, RawL3Entry::TYPE);
            entry.0.set_value(0b1_u64, RawL3Entry::VALID);
        }

        KernPageTable{
            0: page_table
        }
    }
}

pub enum PagePerm {
    RW,
    RO,
    RWX,
}

pub struct UserPageTable(Box<PageTable>);

impl UserPageTable {
    /// Returns a new `UserPageTable` containing a `PageTable` created with
    /// `USER_RW` permission.
    pub fn new() -> UserPageTable {
        UserPageTable{
            0: PageTable::new(USER_RW)
        }
    }

    /// Allocates a page and set an L3 entry translates given virtual address to the
    /// physical address of the allocated page. Returns the allocated page.
    ///
    /// # Panics
    /// Panics if the virtual address is lower than `USER_IMG_BASE`.
    /// Panics if the virtual address has already been allocated.
    /// Panics if allocator fails to allocate a page.
    ///
    /// TODO. use Result<T> and make it failurable
    /// TODO. use perm properly
    pub fn alloc(&mut self, va: VirtualAddr, _perm: PagePerm) -> &mut [u8] {
        if (va.as_ptr() as usize) < USER_IMG_BASE {
            panic!("invalid virtual address");
        }

        let entry =  self.get_entry_mut(va);
        if entry.is_valid() {
            panic!("entry has already been allocated");
        }

        let page = unsafe { ALLOCATOR.alloc(Page::layout()) };

        entry.0.set_masked(page as u64, RawL3Entry::ADDR);
        //entry.0.set_masked(A)

        return unsafe {core::slice::from_raw_parts_mut(page, PAGE_SIZE)};
    }
}

impl Deref for KernPageTable {
    type Target = PageTable;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for UserPageTable {
    type Target = PageTable;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for KernPageTable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl DerefMut for UserPageTable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// FIXME: Implement `Drop` for `UserPageTable`.
impl Drop for UserPageTable {
    fn drop(&mut self) {
        for entry in self.into_iter() {
            if !entry.is_valid() {
                break;
            }

            let address = entry.0.get_masked(RawL3Entry::ADDR) << 16;

            unsafe {
                ALLOCATOR.dealloc(address as *mut u8, Page::layout())
            }
        }
    }
}

impl fmt::Debug for UserPageTable {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("UserPageTable")
            .finish()
    }
}

// FIXME: Implement `fmt::Debug` as you need.
