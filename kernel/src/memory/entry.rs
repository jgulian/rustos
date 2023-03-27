use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::fmt;
use core::fmt::Formatter;
use core::sync::atomic::AtomicUsize;
use aarch64::{EntryCow, EntryNx, EntryPerm, RawL3Entry};
use crate::memory::{PagePermissions, PageTable, PageTableResult, VirtualAddress};
use crate::param::PAGE_ALIGN;

#[derive(Copy, Clone)]
pub struct L3Entry(RawL3Entry);

impl L3Entry {
    /// Returns a new `L3Entry`.
    fn empty() -> Self {
        Self(RawL3Entry::new(0))
    }

    fn new(entry_valid: u64, entry_type: u64) -> Self {

    }

    /// Returns `true` if the L3Entry is valid and `false` otherwise.
    pub fn is_valid(&self) -> bool {
        self.0.get_value(RawL3Entry::VALID) > 0
    }

    pub fn address(&self) -> usize {
        (self.0.get_value(RawL3Entry::ADDR) as usize) << PAGE_ALIGN
    }

    pub fn permissions(&self) -> PagePermissions {
        match (self.0.get_value(RawL3Entry::AP), self.0.get_value(RawL3Entry::XN)) {
            (EntryPerm::USER_RW, EntryNx::Nx) => PagePermissions::RW,
            (EntryPerm::USER_RO, EntryNx::Nx) => PagePermissions::RO,
            (EntryPerm::USER_RW, EntryNx::Ex) => PagePermissions::RWX,
            (EntryPerm::USER_RO, EntryNx::Ex) => PagePermissions::RX,
            _ => {
                panic!("Invalid page permission type.")
            }
        }
    }

    pub fn set_permissions(&mut self, permissions: PagePermissions) {
        match permissions {
            PagePermissions::RW | PagePermissions::RWX => {
                self.0.set_value(EntryPerm::USER_RW, RawL3Entry::AP);
            }
            PagePermissions::RO | PagePermissions::RX => {
                self.0.set_value(EntryPerm::USER_RO, RawL3Entry::AP);
            }
        }

        match permissions {
            PagePermissions::RWX | PagePermissions::RX => {
                self.0.set_value(EntryNx::Ex, RawL3Entry::XN);
            }
            PagePermissions::RW | PagePermissions::RO => {
                self.0.set_value(EntryNx::Nx, RawL3Entry::XN);
            }
        }
    }

    pub fn set_cow(&mut self, cow: bool) {
        match cow {
            true => self.0.set_value(EntryCow::Cow, RawL3Entry::COW),
            false => self.0.set_value(EntryCow::Own, RawL3Entry::COW),
        };
    }

    pub fn is_cow(&self) -> bool {
        self.0.get_value(RawL3Entry::COW) > 0
    }
}

impl From<u64> for L3Entry {
    fn from(v: u64) -> Self {
        L3Entry(RawL3Entry::new(v))
    }
}

impl From<RawL3Entry> for L3Entry {
    fn from(value: RawL3Entry) -> Self {
        Self(value)
    }
}

impl From<L3Entry> for RawL3Entry {
    fn from(val: L3Entry) -> Self {
        val.0
    }
}

impl Into<RawL3Entry> for L3Entry {
    fn into(self) -> RawL3Entry {
        self.0
    }
}

impl fmt::Display for L3Entry {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl fmt::Debug for L3Entry {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("L3Entry")
            .field("addr", &self.0.get_value(RawL3Entry::ADDR))
            .field("af", &self.0.get_value(RawL3Entry::AF))
            .field("sh", &self.0.get_value(RawL3Entry::SH))
            .field("ap", &self.0.get_value(RawL3Entry::AP))
            .field("attr", &self.0.get_value(RawL3Entry::ATTR))
            .field("type", &self.0.get_value(RawL3Entry::TYPE))
            .field("valid", &self.0.get_value(RawL3Entry::VALID))
            .finish()
    }
}

struct PageTableEntryCollection(Box<PageTable>, Vec<PageTableEntry>);

impl PageTableEntryCollection {
    fn allocate_entry(&mut self, location: VirtualAddress, entry: L3Entry) -> PageTableResult<()> {

        Ok(())
    }

    fn acquire_entry_at(&mut self, location: VirtualAddress) -> PageTableResult<()> {
        Ok(())
    }
}

impl Default for PageTableEntryCollection {
    fn default() -> Self {
        //TODO: this shouldn't be user_rw?
        Self(PageTable::new(EntryPerm::USER_RW), Vec::default())
    }
}

enum PageTableEntry {
    Owned(L3Entry),
    CopyOnWrite(L3Entry, Arc<AtomicUsize>),
}

impl PageTableEntry {
    fn l3_entry(&self) -> L3Entry {
        match self {
            PageTableEntry::Owned(l3_entry) => *l3_entry,
            PageTableEntry::CopyOnWrite(l3_entry, _) => *l3_entry,
        }
    }
}