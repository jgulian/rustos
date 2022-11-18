use shim::io;
use shim::io::SeekFrom;
use core::result::Result::Ok;
use crate::headers::{FileHeader, ProgramHeader, SectionHeader};
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt::{Debug, Formatter};
use log::info;

pub trait File: io::Read + io::Seek {}

impl<T> File for T where T: io::Read + io::Seek {}

pub struct Elf {
    file: Box<dyn File>,
    header: FileHeader,
}

impl Elf {
    pub fn new(mut file: Box<dyn File>) -> io::Result<Self> {
        let mut head_buffer = [0u8; 0x40];
        file.read_exact(&mut head_buffer)?;
        let header = unsafe {
            core::mem::transmute::<[u8; 0x40], FileHeader>(head_buffer)
        };

        Ok(Elf {
            file,
            header,
        })
    }

    pub fn read_headers(&mut self) -> io::Result<()> {
        let mut result: Vec<ProgramHeader> = Vec::new();

        for i in 0..self.header.program_table_entries {
            let seek_location = self.header.program_table_offset + (i * self.header.program_entry_size) as u64;
            self.file.seek(SeekFrom::Start(seek_location))?;

            let mut buffer = [0u8; 0x38];
            self.file.read_exact(&mut buffer)?;
            let header = unsafe {core::mem::transmute::<[u8; 0x38], ProgramHeader>(buffer)};

            info!("header: {:?}", header);

            result.push(header);
        }

        Ok(())
    }

    pub(crate) fn read_sections(&mut self) -> io::Result<Vec<SectionHeader>> {
        let mut result: Vec<SectionHeader> = Vec::new();

        for i in 0..self.header.section_table_entries {
            let seek_location = self.header.section_table_offset + (i * self.header.section_entry_size) as u64;
            self.file.seek(SeekFrom::Start(seek_location))?;

            let mut buffer = [0u8; 0x40];
            self.file.read_exact(&mut buffer)?;
            let section = unsafe {core::mem::transmute::<[u8; 0x40], SectionHeader>(buffer)};

            result.push(section);
        }

        Ok(result)
    }
}

impl Debug for Elf {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        f.debug_struct("Elf")
            .field("header", &self.header)
            .finish()
    }
}