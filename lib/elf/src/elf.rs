use shim::io;
use shim::io::SeekFrom;
use core::result::Result::Ok;
use crate::{FileHeader, ProgramHeader, SectionHeader};
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt::{Debug, Formatter};
use log::info;
use crate::reader::ElfReader;
use crate::File;

pub struct Elf {
    reader: ElfReader,
    header: FileHeader,
}

impl Elf {
    pub fn new(mut file: Box<dyn File>) -> io::Result<Self> {
        let mut reader = ElfReader::new(file);
        let header = reader.read_file_header()?;

        Ok(Elf {
            reader,
            header,
        })
    }

    pub fn headers(&mut self) -> io::Result<Vec<ProgramHeader>> {
        (0..self.header.e_phnum).map(|i| {
            let location = self.header.e_phoff + (i * self.header.e_phentsize) as u64;
            self.reader.read_program_header(location)
        }).collect()
    }
}