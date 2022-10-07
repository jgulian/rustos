use crate::vfat::*;
use core::fmt;

use self::Status::*;

#[derive(Debug, PartialEq)]
pub enum Status {
    /// The FAT entry corresponds to an unused (free) cluster.
    Free,
    /// The FAT entry/cluster is reserved.
    Reserved,
    /// The FAT entry corresponds to a valid data cluster. The next cluster in
    /// the chain is `Cluster`.
    Data(Cluster),
    /// The FAT entry corresponds to a bad (disk failed) cluster.
    Bad,
    /// The FAT entry corresponds to a valid data cluster. The corresponding
    /// cluster is the last in its chain.
    Eoc(u32),
}

#[repr(C, packed)]
pub struct FatEntry(pub u32);

impl FatEntry {
    /// Returns the `Status` of the FAT entry `self`.
    pub fn status(&self) -> Status {
        let status_bits = self.0 & 0x0FFF_FFFF;

        if status_bits == 0 {
            Status::Free
        } else if status_bits == 1 {
        } else if 2 <= status_bits && status_bits <= 0x0FFF_FFEF {
            Status::Data(status_bits)
        } else if status_bits == 0x0FFF_FFF7 {
            Status::Bad
        } else if 0x0FFF_FFF8 <= status_bits && status_bits <= 0x0FFF_FFFF {
            Status::Eoc(status_bits)
        } else {
            Status::Reserved
        }
    }
}

impl fmt::Debug for FatEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("FatEntry")
            .field("value", &{ self.0 })
            .field("status", &self.status())
            .finish()
    }
}
