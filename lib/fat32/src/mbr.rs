use core::{fmt, mem};
use core::fmt::{Debug, Write};
use shim::const_assert_size;
use shim::io;

use crate::traits::BlockDevice;

/// Sector includes bottom two bits of sector.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct CHS {
    header: u8,
    sector: u8,
    cylinder: u8,
}

impl CHS {
    fn header(&self) -> u8 {
        self.header
    }

    fn sector(&self) -> u8 {
        self.sector >> 2
    }

    fn cylinder(&self) -> u16 {
        (self.cylinder as u16) & ((self.sector as u16 & 0b11) << 8)
    }
}

impl Debug for CHS {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CHS")
            .field("cylinder", self.cylinder)
            .field("header", self.header)
            .field("sector", self.sector)
            .finish()
    }
}

const_assert_size!(CHS, 3);

#[repr(C, packed)]
pub struct PartitionEntry {
    pub boot_indicator: u8,
    pub starting_chs: CHS,
    pub partition_type: u8,
    pub ending_chs: CHS,
    pub relative_sector: u32,
    pub total_sectors: u32,
}

impl Debug for PartitionEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PartitionEntry")
            .field("boot_indicator", self.boot_indicator)
            .field("starting_chs", self.starting_chs)
            .field("partition_type", self.partition_type)
            .field("ending_chs", self.ending_chs)
            .field("relative_sector", self.relative_sector)
            .field("total_sectors", self.total_sectors)
            .finish()
    }
}

const_assert_size!(PartitionEntry, 16);

/// The master boot record (MBR).
#[repr(C, packed)]
pub struct MasterBootRecord {
    pub bootstrap: [u8; 436],
    pub disk_id: [u8; 10],
    pub partition_table: [PartitionEntry; 4],
    pub valid_bootsector: u16,
}

const_assert_size!(MasterBootRecord, 512);

#[derive(Debug)]
pub enum Error {
    /// There was an I/O error while reading the MBR.
    Io(io::Error),
    /// Partiion `.0` (0-indexed) contains an invalid or unknown boot indicator.
    UnknownBootIndicator(u8),
    /// The MBR magic signature was invalid.
    BadSignature,
}

impl MasterBootRecord {
    /// Reads and returns the master boot record (MBR) from `device`.
    ///
    /// # Errors
    ///
    /// Returns `BadSignature` if the MBR contains an invalid magic signature.
    /// Returns `UnknownBootIndicator(n)` if partition `n` contains an invalid
    /// boot indicator. Returns `Io(err)` if the I/O error `err` occured while
    /// reading the MBR.
    pub fn from<T: BlockDevice>(mut device: T) -> Result<MasterBootRecord, Error> {
        let buffer: [u8; 512] = [0; 512];
        device.read_sector(512, &mut buffer).map_err(|e| Error::Io(e))?;
        let master_boot_record: MasterBootRecord = unsafe {
            mem::transmute(buffer)
        };

        if master_boot_record.valid_bootsector != 0x55AA {
            return Err(Error::BadSignature);
        }

        for partition in master_boot_record.partition_table.iter() {
            if partition.boot_indicator != 0 && partition.boot_indicator != 0x80 {
                return Err(Error::UnknownBootIndicator(partition.boot_indicator));
            }
        }

        Ok(master_boot_record)
    }
}

impl Debug for MasterBootRecord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MasterBootRecord")
            .field("bootstrap", self.bootstrap)
            .field("disk_id", self.disk_id)
            .field("partition_table", self.partition_table)
            .field("valid_bootsector", self.valid_bootsector)
            .finish()
    }
}