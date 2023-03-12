use core::fmt;

use format::Format;
use filesystem::device::BlockDevice;
use filesystem::error::FilesystemError;
use filesystem::partition::BlockPartition;
use shim::const_assert_size;

//TODO: go through all repr C and remove
#[derive(Format)]
pub struct BiosParameterBlock {
    pub jump_instructions: [u8; 3],
    pub oem_identifier: [u8; 8],
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sectors: u16,
    pub number_of_fats: u8,
    pub max_num_of_dirs: u16,
    pub total_logical_sectors: u16,
    pub media_desciptor_type: u8,
    pub sectors_per_fat_one: u16,
    pub sectors_per_track: u16,
    pub number_of_heads: u16,
    pub number_of_hidden_sectors: u32,
    pub total_logical_sectors_extended: u32,
    pub sectors_per_fat_two: u32,
    pub flags: u16,
    pub version: u16,
    pub root_cluster: u32,
    pub sector_of_fsinfo: u16,
    pub sector_of_backup: u16,
    pub reserved: [u8; 12],
    pub drive_number: u8,
    pub nt_flags: u8,
    pub signature: u8,
    pub serial_number: u32,
    pub label_string: [u8; 11],
    pub system_identifier: [u8; 8],
    pub boot_code: [u8; 420],
    pub bp_signature: [u8; 2],
}

const_assert_size!(BiosParameterBlock, 512);

impl TryFrom<&mut BlockPartition> for BiosParameterBlock {
    type Error = FilesystemError;

    /// Reads the FAT32 extended BIOS parameter block from sector `sector` of
    /// device `device`.
    ///
    /// # Errors
    ///
    /// If the EBPB signature is invalid, returns an error of `BadSignature`.
    fn try_from(value: &mut BlockPartition) -> Result<Self, Self::Error> {
        let mut buffer: [u8; 512] = [0; 512];
        value.read_block(0, &mut buffer)?;

        use format::Format;
        let bios_parameter_block: BiosParameterBlock = BiosParameterBlock::load_slice(&buffer)?;

        if bios_parameter_block.bp_signature[0] != 0x55 ||
            bios_parameter_block.bp_signature[1] != 0xAA {
            return Err(FilesystemError::BadSignature);
        }

        Ok(bios_parameter_block)
    }
}

impl fmt::Debug for BiosParameterBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MasterBootRecord")
            .field("oem_identifier", &self.oem_identifier)
            .field("bytes_per_sector", &self.bytes_per_sector)
            .field("sectors_per_cluster", &self.sectors_per_cluster)
            .field("reserved_sectors", &self.reserved_sectors)
            .field("number_of_fats", &self.number_of_fats)
            .field("max_num_of_dirs", &self.max_num_of_dirs)
            .field("total_logical_sectors", &self.total_logical_sectors)
            .field("media_desciptor_type", &self.media_desciptor_type)
            .field("sectors_per_fat_one", &self.sectors_per_fat_one)
            .field("sectors_per_track", &self.sectors_per_track)
            .field("number_of_heads", &self.number_of_heads)
            .field("number_of_hidden_sectors", &self.number_of_hidden_sectors)
            .field("total_logical_sectors_extended", &self.total_logical_sectors_extended)
            .field("sectors_per_fat_two", &self.sectors_per_fat_two)
            .field("flags", &self.flags)
            .field("version", &self.version)
            .field("root_cluster", &self.root_cluster)
            .field("sector_of_fsinfo", &self.sector_of_fsinfo)
            .field("sector_of_backup", &self.sector_of_backup)
            .field("drive_number", &self.drive_number)
            .field("signature", &self.signature)
            .field("serial_number", &self.serial_number)
            .field("label_string", &self.label_string)
            .field("system_identifier", &self.system_identifier)
            .field("boot_code", &self.boot_code)
            .field("bp_signature", &self.bp_signature)
            .finish()
    }
}