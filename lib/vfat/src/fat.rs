use crate::cluster::Cluster;
use core::fmt;

use self::Status::*;

#[derive(Debug, PartialEq)]
pub(crate) enum Status {
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

impl Status {
    pub(crate) fn new_eoc() -> Self {
        Eoc(0x0FFF_FFF8)
    }
}

#[derive(Default, Copy, Clone)]
pub struct FatEntry(u32);

impl From<Status> for FatEntry {
    fn from(value: Status) -> Self {
        match value {
            Free => FatEntry(0),
            Reserved => FatEntry(0x0FFF_FFF6),
            Data(cluster) => FatEntry(cluster.into()),
            Bad => FatEntry(0x0FFF_FFF7),
            Eoc(status_bits) => FatEntry(status_bits),
        }
    }
}

impl From<[u8; 4]> for FatEntry {
    fn from(value: [u8; 4]) -> Self {
        Self(u32::from_le_bytes(value))
    }
}

impl Into<[u8; 4]> for FatEntry {
    fn into(self) -> [u8; 4] {
        self.0.to_le_bytes()
    }
}

impl FatEntry {
    /// Returns the `Status` of the FAT entry `self`.
    pub(crate) fn status(&self) -> Status {
        let status_bits = self.0 & 0x0FFF_FFFF;

        if status_bits == 0 {
            Free
        } else if 2 <= status_bits && status_bits <= 0x0FFF_FFEF {
            Data(Cluster::from(status_bits))
        } else if status_bits == 0x0FFF_FFF7 {
            Bad
        } else if 0x0FFF_FFF8 <= status_bits && status_bits <= 0x0FFF_FFFF {
            Eoc(status_bits)
        } else {
            Reserved
        }
    }

    pub(crate) fn is_free(&self) -> bool {
        self.status() == Free
    }

    pub(crate) fn find(
        cluster: Cluster,
        fat_start_sector: u64,
        bytes_per_sector: u64,
    ) -> (u64, usize) {
        let block = fat_start_sector + cluster.offset() as u64 / bytes_per_sector;
        let offset = (cluster.offset() as u64 % bytes_per_sector) as usize;
        (block, offset)
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
