use shim::io;
use crate::BlockDevice;
use format::Format;
use format_derive::Format;
use shim::io::{Read, Write, Seek, Result, Cursor};

#[derive(Copy, Clone, Format)]
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

#[derive(Copy, Clone, Format)]
pub struct PartitionEntry {
    pub boot_indicator: u8,
    pub starting_chs: CHS,
    pub partition_type: u8,
    pub ending_chs: CHS,
    pub relative_sector: u32,
    pub total_sectors: u32,
}

#[derive(Copy, Clone, Format)]
pub struct MasterBootRecord {
    pub bootstrap: [u8; 436],
    pub disk_id: [u8; 10],
    pub partition_table: [PartitionEntry; 4],
    pub valid_boot_sector: [u8; 2],
}

#[derive(Debug)]
pub enum Error {
    /// There was an I/O error while reading the MBR.
    Io(io::Error),
    /// Partition `.0` (0-indexed) contains an invalid or unknown boot indicator.
    UnknownBootIndicator(u8),
    /// The MBR magic signature was invalid.
    BadSignature,
}

//TODO: use MBRError again

impl MasterBootRecord {
    /// Reads and returns the master boot record (MBR) from `device`.
    ///
    /// # Errors
    ///
    /// Returns `BadSignature` if the MBR contains an invalid magic signature.
    /// Returns `UnknownBootIndicator(n)` if partition `n` contains an invalid
    /// boot indicator. Returns `Io(err)` if the I/O error `err` occured while
    /// reading the MBR.
    pub fn from<T: BlockDevice>(mut device: T) -> Result<MasterBootRecord> {
        let mut buffer: [u8; 512] = [0; 512];
        //device.read_sector(0, &mut buffer).map_err(|e| Error::Io(e))?;
        device.read_sector(0, &mut buffer).map_err(|e|
            io::Error::from(io::ErrorKind::Interrupted))?;

        let master_boot_record = MasterBootRecord::load_readable_seekable(&mut Cursor::new(buffer.as_mut_slice()))?;

        if master_boot_record.valid_boot_sector[0] != 0x55 ||
            master_boot_record.valid_boot_sector[1] != 0xAA {
            return Err(io::Error::from(io::ErrorKind::Interrupted));
        }

        for (i, partition) in master_boot_record.partition_table.iter().enumerate() {
            if partition.boot_indicator != 0 && partition.boot_indicator != 0x80 {
                //return Err(Error::UnknownBootIndicator(i as u8));
                return Err(io::Error::from(io::ErrorKind::Interrupted));
            }
        }

        Ok(master_boot_record)
    }
}