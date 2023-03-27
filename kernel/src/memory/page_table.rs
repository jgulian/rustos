use alloc::boxed::Box;
use alloc::fmt;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::alloc::{GlobalAlloc, Layout};
use core::fmt::Formatter;
use core::iter::{Chain, Map};
use core::ops::{Deref, DerefMut, Sub};

use core::slice;
use core::slice::Iter;
use core::sync::atomic::AtomicUsize;

use aarch64::vmsa::*;
use allocator::util::{align_down, align_up};
use kernel_api::{OsError, OsResult};
use shim::{const_assert_size, io, ioerr};

use crate::{ALLOCATOR, VMM};

use crate::memory::{PhysicalAddress, VirtualAddress};
use crate::memory::entry::L3Entry;
use crate::param::*;

#[repr(C)]
pub struct Page([u8; PAGE_SIZE]);

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

impl L2PageTable {
    /// Returns a new `L2PageTable`
    fn new() -> L2PageTable {
        L2PageTable {
            entries: [RawL2Entry::new(0); 8192],
        }
    }

    /// Returns a `PhysicalAddr` of the pagetable.
    pub fn as_ptr(&self) -> PhysicalAddress {
        PhysicalAddress::from(self as *const Self as usize)
    }
}

#[repr(C)]
#[repr(align(65536))]
pub struct L3PageTable {
    pub entries: [RawL3Entry; 8192],
}
const_assert_size!(L3PageTable, PAGE_SIZE);

impl L3PageTable {
    /// Returns a new `L3PageTable`.
    fn new() -> L3PageTable {
        L3PageTable {
            entries: [RawL3Entry::new(0); 8192],
        }
    }

    /// Returns a `PhysicalAddr` of the pagetable.
    pub fn as_ptr(&self) -> PhysicalAddress {
        PhysicalAddress::from(self as *const Self as usize)
    }
}

#[repr(C)]
#[repr(align(65536))]
pub struct PageTable {
    pub l2: L2PageTable,
    pub l3: [L3PageTable; 3],
}

impl PageTable {
    /// Returns a new `Box` containing `PageTable`.
    /// Entries in L2PageTable should be initialized properly before return.
    pub(super) fn new(perm: u64) -> Box<PageTable> {
        let mut page_table = Box::new(PageTable {
            l2: L2PageTable::new(),
            l3: [L3PageTable::new(), L3PageTable::new(), L3PageTable::new()],
        });

        for i in 0..3 {
            let mut page_entry = RawL2Entry::new(0);
            page_entry.set_value(page_table.l3[i].as_ptr().as_u64() >> PAGE_ALIGN, RawL2Entry::ADDR);
            page_entry.set_value(1, RawL2Entry::AF);
            page_entry.set_value(EntrySh::ISh, RawL2Entry::AP);
            page_entry.set_value(perm, RawL2Entry::SH);
            page_entry.set_value(EntryAttr::Mem, RawL2Entry::ATTR);
            page_entry.set_value(EntryType::Table, RawL2Entry::TYPE);
            page_entry.set_value(EntryValid::Valid, RawL2Entry::VALID);

            page_table.l2.entries[i] = page_entry;
        }

        page_table
    }

    /// Returns the (L2index, L3index) extracted from the given virtual address.
    /// L2index should be smaller than the number of L3PageTable.
    ///
    /// # Panics
    ///
    /// Panics if the virtual address is not properly aligned to page size.
    /// Panics if extracted L2index exceeds the number of L3PageTable.
    fn locate(va: VirtualAddress) -> (usize, usize) {
        if va.as_usize() % PAGE_SIZE != 0 {
            panic!("virtual address is unaligned")
        }

        let l2_index = va.level2_index();
        let l3_index = va.level3_index();
        if 3 <= l2_index {
            panic!("l2 index out of bounds");
        }

        (l2_index as usize, l3_index as usize)
    }

    /// Returns `true` if the L3entry indicated by the given virtual address is valid.
    /// Otherwise, `false` is returned.
    pub fn is_valid(&self, va: VirtualAddress) -> bool {
        let (l2_index, l3_index) = PageTable::locate(va);
        self.l3[l2_index].entries[l3_index].is_valid()
    }

    /// Returns `true` if the L3entry indicated by the given virtual address is invalid.
    /// Otherwise, `true` is returned.
    pub fn is_invalid(&self, va: VirtualAddress) -> bool {
        !self.is_valid(va)
    }

    pub fn get_entry(&self, va: VirtualAddress) -> L3Entry {
        let (l2, l3) = PageTable::locate(va);
        L3Entry::from(self.l3[l2].entries[l3])
    }

    /// Set the given RawL3Entry `entry` to the L3Entry indicated by the given virtual
    /// address.
    pub fn set_entry(&mut self, va: VirtualAddress, entry: L3Entry) -> &mut Self {
        let (l2, l3) = PageTable::locate(va);
        self.l3[l2].entries[l3].0 = entry.into();
        self
    }

    /// Returns a base address of the pagetable. The returned `PhysicalAddr` value
    /// will point the start address of the L2PageTable.
    pub fn get_baddr(&self) -> PhysicalAddress {
        self.l2.as_ptr()
    }
}

impl<'a> IntoIterator for &'a PageTable {
    type Item = L3Entry;
    type IntoIter = Map<Chain<Iter<'a, RawL3Entry>, Iter<'a, RawL3Entry>>, fn(&RawL3Entry) -> L3Entry>;

    fn into_iter(self) -> Self::IntoIter {
        self.l3[0].entries.iter().chain(self.l3[1].entries.iter())
            .map(|l3_entry| L3Entry::from(l3_entry))
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
        let mut page_table = PageTable::new(EntryPerm::KERN_RW);

        let page_start = 0;
        let page_end = align_down(0x3c000000, PAGE_ALIGN);
        let page_count = (page_end - page_start) / PAGE_SIZE;

        for i in 0..page_count {
            let address = page_start + PAGE_SIZE * i;
            let mut entry = RawL3Entry::new(0);
            entry.set_value(address as u64 >> PAGE_ALIGN, RawL3Entry::ADDR);
            entry.set_value(EntrySh::ISh, RawL3Entry::SH);
            entry.set_value(EntryPerm::KERN_RW, RawL3Entry::AP);
            entry.set_value(EntryAttr::Mem, RawL3Entry::ATTR);
            entry.set_value(EntryType::Table, RawL3Entry::TYPE);
            entry.set_value(EntryValid::Valid, RawL3Entry::VALID);
            entry.set_value(0b1_u64, RawL3Entry::AF);
            page_table.set_entry(VirtualAddress::from(address), L3Entry::from(entry));
        }

        let device_start = align_down(IO_BASE, PAGE_ALIGN);
        let device_end = align_up(IO_BASE_END, PAGE_ALIGN);
        let device_page_count = (device_end - device_start) / PAGE_SIZE;

        for i in 0..device_page_count {
            let address = device_start + PAGE_SIZE * i;
            let mut entry = RawL3Entry::new(0);
            entry.set_value(address as u64 >> PAGE_ALIGN, RawL3Entry::ADDR);
            entry.set_value(EntrySh::OSh, RawL3Entry::SH);
            entry.set_value(EntryPerm::KERN_RW, RawL3Entry::AP);
            entry.set_value(EntryAttr::Dev, RawL3Entry::ATTR);
            entry.set_value(EntryType::Table, RawL3Entry::TYPE);
            entry.set_value(EntryValid::Valid, RawL3Entry::VALID);
            entry.set_value(0b1_u64, RawL3Entry::AF);
            page_table.set_entry(VirtualAddress::from(address), L3Entry::from(entry));
        }

        KernPageTable(page_table)
    }
}

pub enum PagePermissions {
    RW,
    RO,
    RWX,
    RX,
}

pub struct UserPageTable(PageTableEntryCollection);

impl UserPageTable {
    /// Returns a new `UserPageTable` containing a `PageTable` created with
    /// `USER_RW` permission.
    pub fn new() -> UserPageTable {
        Self(PageTableEntryCollection::default())
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
    pub fn alloc(&mut self, mut va: VirtualAddress, permissions: PagePermissions) -> &mut [u8] {
        if va.as_usize() < USER_IMG_BASE {
            panic!("invalid virtual address");
        }

        va = va.sub(VirtualAddress::from(USER_IMG_BASE));

        if self.0.is_valid(va) {
            panic!("entry has already been allocated");
        }

        let page = unsafe { ALLOCATOR.alloc(Page::layout()) };
        let address = page as u64;

        let mut raw_l3_entry = RawL3Entry::new(0);
        raw_l3_entry.set_value(address >> PAGE_ALIGN, RawL3Entry::ADDR);
        raw_l3_entry.set_value(EntrySh::ISh, RawL3Entry::SH);
        raw_l3_entry.set_value(EntryAttr::Mem, RawL3Entry::ATTR);
        raw_l3_entry.set_value(EntryType::Table, RawL3Entry::TYPE);
        raw_l3_entry.set_value(EntryValid::Valid, RawL3Entry::VALID);
        raw_l3_entry.set_value(0b1_u64, RawL3Entry::AF);

        let mut l3_entry = L3Entry::from(raw_l3_entry);
        l3_entry.set_permissions(permissions);
        self.0.set_entry(va, l3_entry);

        return unsafe { core::slice::from_raw_parts_mut(page, PAGE_SIZE) };
    }

    pub fn cow(&mut self, mut virtual_address: VirtualAddress, other_l3_entry: &mut L3Entry) -> OsResult<()> {
        if virtual_address.as_usize() < USER_IMG_BASE {
            panic!("invalid virtual address");
        }

        virtual_address = virtual_address.sub(VirtualAddress::from(USER_IMG_BASE));

        if self.0.is_valid(virtual_address) {
            panic!("entry has already been allocated");
        }

        let address = other_l3_entry.address() as u64;
        let physical_address = PhysicalAddress::from(address);

        match other_l3_entry.permissions() {
            PagePermissions::RW => {
                other_l3_entry.set_permissions(PagePermissions::RO);
            },
            PagePermissions::RWX => {
                other_l3_entry.set_permissions(PagePermissions::RX);
            },
            _ => {},
        }

        if !other_l3_entry.is_cow() {
            other_l3_entry.set_cow(true);
            VMM.pin_frame(physical_address);
        }

        let l3_entry = *other_l3_entry;
        self.0.set_entry(virtual_address, l3_entry);
        VMM.pin_frame(physical_address);

        Ok(())
    }

    pub fn remove_cow(&mut self, mut virtual_address: VirtualAddress, pid: u64) -> OsResult<()> {
        if virtual_address.as_usize() < USER_IMG_BASE {
            return Err(OsError::BadAddress);
        }

        virtual_address = virtual_address.sub(VirtualAddress::from(USER_IMG_BASE));
        let mut l3_entry = self.get_entry(virtual_address);

        if !l3_entry.is_valid() || !l3_entry.is_cow() {
            return Err(OsError::Unknown);
        }

        let permissions = match l3_entry.permissions() {
            PagePermissions::RW => PagePermissions::RW,
            PagePermissions::RO => PagePermissions::RW,
            PagePermissions::RWX => PagePermissions::RWX,
            PagePermissions::RX => PagePermissions::RWX,
        };

        l3_entry.set_permissions(permissions);
        l3_entry.set_cow(false);

        let physical_address = PhysicalAddress::from(l3_entry.address());
        let pin_count = VMM.get_frame_pin_count(physical_address);
        let new_address = if pin_count > 1 {
            let (new_address, destination_page) = unsafe {
                let ptr = ALLOCATOR.alloc(Page::layout());
                (ptr as usize, slice::from_raw_parts_mut(ptr, PAGE_SIZE))
            };

            let source_page = unsafe {
                let ptr = physical_address.as_ptr();
                slice::from_raw_parts(ptr, PAGE_SIZE)
            };

            destination_page.copy_from_slice(source_page);


            if cfg!(feature = "monitor_lab2") {
                info!("{}: copy at {:x}", pid, virtual_address.as_usize() + USER_IMG_BASE);
            }

            new_address
        } else {
            physical_address.as_usize()
        } as u64;

        VMM.unpin_frame(physical_address);

        l3_entry.0.set_value(new_address >> PAGE_ALIGN, RawL3Entry::ADDR);
        self.set_entry(virtual_address, l3_entry);

        Ok(())
    }

    pub fn translate(&self, virtual_address: VirtualAddress) -> io::Result<PhysicalAddress> {
        let page_aligned = virtual_address.page_aligned();
        let (l2_index, l3_index) = PageTable::locate(page_aligned);
        let l3_entry = &self.l3[l2_index].entries[l3_index];
        if l3_entry.is_valid() {
            let page_address = l3_entry.0.get_value(RawL3Entry::ADDR) << PAGE_ALIGN;
            Ok(PhysicalAddress::from(page_address + virtual_address.offset()))
        } else {
            ioerr!(AddrNotAvailable)
        }
    }

    pub fn allocated(&mut self) -> impl Iterator<Item=(VirtualAddress, &mut L3Entry)> {
        self.l3.iter_mut().enumerate().flat_map(|(i, table)| {
            table.entries.iter_mut().enumerate().filter_map(move |(j, l3_entry)| {
                let virtual_address = VirtualAddress::from(
                    USER_IMG_BASE + ((i & ((1 << 14) - 1)) << 29 | (j & ((1 << 14) - 1)) << 16));
                if l3_entry.is_valid() {
                    Some((virtual_address, l3_entry))
                } else {
                    None
                }
            })
        })
    }
}

//TODO: remove?
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
        self.allocated().for_each(|(_, l3_entry)| {
            let mut physical_address = PhysicalAddress::from(l3_entry.address());

            if l3_entry.is_cow() {
                VMM.unpin_frame(physical_address);
            } else {
                unsafe {
                    ALLOCATOR.dealloc(physical_address.as_mut_ptr(), Page::layout());
                }
            }
        });
    }
}

impl fmt::Display for UserPageTable {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for l3_entry in self.into_iter() {
            if l3_entry.is_valid() {
                f.write_fmt(format_args!("{}\n", l3_entry))?;
            }
        }
        Ok(())
    }
}

impl fmt::Debug for UserPageTable {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for l3_entry in self.into_iter() {
            if l3_entry.is_valid() {
                f.write_fmt(format_args!("{}\n", l3_entry))?;
            }
            f.write_fmt(format_args!("{}\n", l3_entry))?;
        }
        Ok(())
    }
}

pub enum PageTableError {
    PageAlreadyAllocated,
}

pub type PageTableResult<T> = Result<T, PageTableError>;