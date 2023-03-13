#[cfg(feature = "no_std")]
use alloc::boxed::Box;
#[cfg(not(feature = "no_std"))]
use std::boxed::Box;
use core::fmt::{Debug, Formatter};
use format::Format;
use crate::device::BlockDevice;
use crate::error::FilesystemError;

#[derive(Copy, Clone, Format)]
pub struct CHS {
    pub header: u8,
    pub sector: u8,
    pub cylinder: u8,
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
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CHS")
            .field("header", &self.header())
            .field("sector", &self.sector())
            .field("cylinder", &self.cylinder())
            .finish()
    }
}

#[derive(Copy, Clone, Debug, Format)]
pub struct PartitionEntry {
    pub boot_indicator: u8,
    pub starting_chs: CHS,
    pub partition_type: u8,
    pub ending_chs: CHS,
    pub relative_sector: u32,
    pub total_sectors: u32,
}

#[derive(Copy, Clone, Debug, Format)]
pub struct MasterBootRecord {
    pub bootstrap: [u8; 436],
    pub disk_id: [u8; 10],
    pub partition_table: [PartitionEntry; 4],
    pub valid_boot_sector: [u8; 2],
}

impl TryFrom<&mut Box<dyn BlockDevice>> for MasterBootRecord {
    type Error = FilesystemError;


    /// Reads and returns the master boot record (MBR) from `device`.
    ///
    /// # Errors
    ///
    /// Returns `BadSignature` if the MBR contains an invalid magic signature.
    /// Returns `UnknownBootIndicator(n)` if partition `n` contains an invalid
    /// boot indicator. Returns `Io(err)` if the I/O error `err` occured while
    /// reading the MBR.
    fn try_from(value: &mut Box<dyn BlockDevice>) -> Result<Self, Self::Error> {
        let mut buffer: [u8; 512] = [0; 512];
        value.read_block(0, &mut buffer)?;

        let master_boot_record = MasterBootRecord::load_slice(&buffer)?;

        if master_boot_record.valid_boot_sector[0] != 0x55 ||
            master_boot_record.valid_boot_sector[1] != 0xAA {
            return Err(FilesystemError::BadSignature);
        }

        for (i, partition) in master_boot_record.partition_table.iter().enumerate() {
            if partition.boot_indicator != 0 && partition.boot_indicator != 0x80 {
                return Err(FilesystemError::UnknownBootIndicator(i as u8));
            }
        }

        Ok(master_boot_record)
    }
}