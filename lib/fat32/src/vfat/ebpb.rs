use core::fmt;
use shim::const_assert_size;

use crate::traits::BlockDevice;
use crate::vfat::Error;

#[repr(C, packed)]
pub struct BiosParameterBlock {
    pub sectors_per_fat: u32,
    pub flags: u16,
    pub version: u16,
    pub cluster_number: u32,
    pub sector_of_fsinfo: u16,
    pub sector_of_backup: u16,
    __reserved_one: [u8; 12],
    pub drive_number: u8,
    __reserved_two: u8,
    pub signature: u8,
    pub serial_number: u32,
    pub label_string: [u8; 11],
    pub system_identifier: [u8; 8],
    pub boot_code: [u8; 420],
    pub bp_signature: u16,
}

const_assert_size!(BiosParameterBlock, 512);

impl BiosParameterBlock {
    /// Reads the FAT32 extended BIOS parameter block from sector `sector` of
    /// device `device`.
    ///
    /// # Errors
    ///
    /// If the EBPB signature is invalid, returns an error of `BadSignature`.
    pub fn from<T: BlockDevice>(mut device: T, sector: u64) -> Result<BiosParameterBlock, Error> {
        let buffer: [u8; 512] = [0; 512];
        device.read_sector(512, &mut buffer).map_err(|e| Error::Io(e))?;
        let bios_parameter_block: BiosParameterBlock = unsafe {
            mem::transmute(buffer)
        };

        if bios_parameter_block.bp_signature  != 0x55AA {
            return Err(Error::BadSignature);
        }

        Ok(bios_parameter_block)
    }
}

impl fmt::Debug for BiosParameterBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MasterBootRecord")
            .field("sectors_per_fat", self.sectors_per_fat)
            .field("flags", self.flags)
            .field("version", self.version)
            .field("cluster_number", self.cluster_number)
            .field("sector_of_fsinfo", self.sector_of_fsinfo)
            .field("sector_of_backup", self.sector_of_backup)
            .field("drive_number", self.drive_number)
            .field("signature", self.signature)
            .field("serial_number", self.serial_number)
            .field("label_string", self.label_string)
            .field("system_identifier", self.system_identifier)
            .field("boot_code", self.boot_code)
            .field("bp_signature", self.bp_signature)
            .finish()
    }
}
