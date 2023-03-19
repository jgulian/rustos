use core::fmt;

use format::Format;
use filesystem::device::BlockDevice;
use filesystem::error::FilesystemError;
use filesystem::partition::BlockPartition;
use shim::const_assert_size;

//TODO: go through all repr C and remove
#[derive(Format)]
pub(crate) struct BiosParameterBlock {
    pub(crate) jump_instructions: [u8; 3],
    pub(crate) oem_identifier: [u8; 8],
    pub(crate) bytes_per_sector: u16,
    pub(crate) sectors_per_cluster: u8,
    pub(crate) reserved_sectors: u16,
    pub(crate) number_of_fats: u8,
    pub(crate) max_num_of_dirs: u16,
    pub(crate) total_logical_sectors: u16,
    pub(crate) media_desciptor_type: u8,
    pub(crate) sectors_per_fat_one: u16,
    pub(crate) sectors_per_track: u16,
    pub(crate) number_of_heads: u16,
    pub(crate) number_of_hidden_sectors: u32,
    pub(crate) total_logical_sectors_extended: u32,
    pub(crate) sectors_per_fat_two: u32,
    pub(crate) flags: u16,
    pub(crate) version: u16,
    pub(crate) root_cluster: u32,
    pub(crate) sector_of_fsinfo: u16,
    pub(crate) sector_of_backup: u16,
    pub(crate) reserved: [u8; 12],
    pub(crate) drive_number: u8,
    pub(crate) nt_flags: u8,
    pub(crate) signature: u8,
    pub(crate) serial_number: u32,
    pub(crate) label_string: [u8; 11],
    pub(crate) system_identifier: [u8; 8],
    pub(crate) boot_code: [u8; 420],
    pub(crate) bp_signature: [u8; 2],
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

impl BiosParameterBlock {
    pub(crate) fn new(sector_size: u16, ) -> Self {
        let mut boot_code = [0u8; 420];
        boot_code[..DEFAULT_BOOT_CODE_HEAD.len()].copy_from_slice(&DEFAULT_BOOT_CODE_HEAD);

        Self {
            jump_instructions: [235, 88, 144],
            oem_identifier: *b"rustmkft",
            bytes_per_sector: sector_size as u16,
            sectors_per_cluster: 1,
            reserved_sectors: 32,
            number_of_fats: 2,
            max_num_of_dirs: 0,
            total_logical_sectors: 0,
            media_desciptor_type: 248,
            sectors_per_fat_one: 0,
            sectors_per_track: 32,
            number_of_heads: 64,
            number_of_hidden_sectors: 0,
            total_logical_sectors_extended: 247952,
            sectors_per_fat_two: 1908,
            flags: 0,
            version: 0,
            root_cluster: 2,
            sector_of_fsinfo: 1,
            sector_of_backup: 6,
            reserved: [0; 12],
            drive_number: 128,
            nt_flags: 0,
            signature: 41,
            serial_number: 3173764726,
            label_string: *b"NO NAME    ",
            system_identifier: *b"FAT32   ",
            boot_code,
            bp_signature: [0x55, 0xaa],
        }
    }
}

const DEFAULT_BOOT_CODE_HEAD: [u8; 129] = *b"\x0e\x1f\xbe\x77\x7c\xac\x22\xc0\x74\x0b\x56\xb4\x0e\
\xbb\x07\x00\xcd\x10\x5e\xeb\xf0\x32\xe4\xcd\x16\xcd\x19\xeb\xfe\
This is not a bootable disk.  Please insert a bootable floppy and\r\npress any key to try again \
... \r\n";