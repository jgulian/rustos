use alloc::boxed::Box;
use core::mem;
use shim::io;
use shim::io::SeekFrom;
use crate::file::{FileHeader, FileHeadIdentity, ObjectFileType, TargetMachineIsa};
use crate::program::{ProgramHeader, ProgramHeaderType};
use crate::section::{SectionHeader, SectionHeaderType};
use crate::util::{Endianness, File, FromBytes};

pub(crate) struct ElfReader {
    file: Box<dyn File>,
    endianness: Endianness,
}

impl ElfReader {
    pub(crate) fn new(file: Box<dyn File>) -> Self {
        ElfReader {
            file,
            endianness: Endianness::Little,
        }
    }

    fn update_endianness(&mut self, endianness: Endianness) {
        self.endianness = endianness;
    }

    pub(crate) fn read_endian<T: FromBytes<N>, const N: usize>(&mut self) -> io::Result<T> {
        let mut buffer = [0u8; N];
        self.file.read(&mut buffer)?;
        match self.endianness {
            Endianness::Big => Ok(T::from_be_bytes(buffer)),
            Endianness::Little => Ok(T::from_le_bytes(buffer)),
            Endianness::Unknown => Ok(T::from_ne_bytes(buffer)),
        }
    }

    pub(crate) fn read_file_header(&mut self) -> io::Result<FileHeader> {
        self.file.seek(SeekFrom::Start(0))?;

        let file_head_identity = FileHeadIdentity::read(&mut self.file)?;
        self.update_endianness(file_head_identity.endianness());

        Ok(FileHeader {
            e_ident: file_head_identity,
            e_type: ObjectFileType::from(self.read_endian::<u16, 2>()?),
            e_machine: TargetMachineIsa::from(self.read_endian::<u16, 2>()?),
            e_version: self.read_endian::<u32, 4>()?,
            e_entry: self.read_endian::<u64, 8>()?,
            e_phoff: self.read_endian::<u64, 8>()?,
            e_shoff: self.read_endian::<u64, 8>()?,
            e_flags: self.read_endian::<u32, 4>()?,
            e_ehsize: self.read_endian::<u16, 2>()?,
            e_phentsize: self.read_endian::<u16, 2>()?,
            e_phnum: self.read_endian::<u16, 2>()?,
            e_shentsize: self.read_endian::<u16, 2>()?,
            e_shnum: self.read_endian::<u16, 2>()?,
            e_shstrndx: self.read_endian::<u16, 2>()?
        })
    }

    pub(crate) fn read_program_header(&mut self, location: u64) -> io::Result<ProgramHeader> {
        self.file.seek(SeekFrom::Start(location))?;

        Ok(ProgramHeader {
            p_type: ProgramHeaderType::from(self.read_endian::<u32, 4>()?),
            p_flags: self.read_endian::<u32, 4>()?,
            p_offset: self.read_endian::<u64, 8>()?,
            p_vaddr: self.read_endian::<u64, 8>()?,
            p_paddr: self.read_endian::<u64, 8>()?,
            p_filesz: self.read_endian::<u64, 8>()?,
            p_memsz: self.read_endian::<u64, 8>()?,
            p_align: self.read_endian::<u64, 8>()?,
        })
    }

    pub(crate) fn read_section_header(&mut self, location: u64) -> io::Result<SectionHeader> {
        self.file.seek(SeekFrom::Start(location))?;

        Ok(SectionHeader {
            sh_name: self.read_endian::<u32, 4>()?,
            sh_type: SectionHeaderType::from(self.read_endian::<u32, 4>()?),
            sh_flags: self.read_endian::<u64, 8>()?,
            sh_addr: self.read_endian::<u64, 8>()?,
            sh_offset: self.read_endian::<u64, 8>()?,
            sh_size: self.read_endian::<u64, 8>()?,
            sh_link: self.read_endian::<u32, 4>()?,
            sh_info: self.read_endian::<u32, 4>()?,
            sh_addralign: self.read_endian::<u64, 8>()?,
            sh_entsize: self.read_endian::<u64, 8>()?,
        })
    }
}

impl io::Seek for ElfReader {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.file.seek(pos)
    }
}