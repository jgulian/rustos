use alloc::boxed::Box;
use crate::aarch64::memory::allocation::{
    DataBlock16Kb, DataBlock2Mb, DataBlock32Mb, DataBlock4Kb, DataBlock512Mb, DataBlock64Kb,
    DataBlockRaw, TranslationTable16Kb, TranslationTable4Kb, TranslationTable64Kb,
    TranslationTableRaw,
};
use crate::primitives::memory::{VirtualMemoryError, VirtualMemoryResult};



use alloc::vec::Vec;
use core::ops::IndexMut;
use crate::aarch64::memory::attributes::EntryAttributes;
use crate::aarch64::memory::table_entry::{TableDescriptor, TranslationTableEntry};

pub(super) struct TranslationTable {
    translation_table: TranslationTableRaw,
    table_descriptors: Vec<(usize, TableDescriptor)>,
}

impl TranslationTable {
    pub(super) fn new(granule_size: GranuleSize) -> Self {
        let translation_table = match granule_size {
            GranuleSize::Kb4 => TranslationTableRaw::Kb4(Box::new(TranslationTable4Kb::default())),
            GranuleSize::Kb16 => {
                TranslationTableRaw::Kb16(Box::new(TranslationTable16Kb::default()))
            }
            GranuleSize::Kb64 => {
                TranslationTableRaw::Kb64(Box::new(TranslationTable64Kb::default()))
            }
        };

        TranslationTable {
            translation_table,
            table_descriptors: Vec::new(),
        }
    }

    pub(super) fn insert(&mut self, offset: usize, table_descriptor: TableDescriptor) -> VirtualMemoryResult<()> {}

    pub(super) fn remove(&mut self, offset: usize) -> VirtualMemoryResult<()> {}

    pub(super) fn len(&self) -> usize {
        match self.translation_table {
            TranslationTableRaw::Kb4(_) => TranslationTable4Kb::len(),
            TranslationTableRaw::Kb16(_) => TranslationTable16Kb::len(),
            TranslationTableRaw::Kb64(_) => TranslationTable64Kb::len(),
        }
    }

    pub(super) fn entry<'a>(&mut self, index: usize) -> VirtualMemoryResult<TranslationTableEntry<'a>> {
        if self.len() <= index {
            return Err(VirtualMemoryError::DescriptorOutOfBounds);
        }

        let raw_entry = match &mut self.translation_table {
            TranslationTableRaw::Kb4(table) =>
                table.index_mut(index),
            TranslationTableRaw::Kb16(table) =>
                table.index_mut(index),
            TranslationTableRaw::Kb64(table) =>
                table.index_mut(index),
        };

        let entry_index = self.table_descriptors
            .iter()
            .enumerate()
            .find_map(|(i, (entry_index, _))| if *entry_index == index { Some(i) } else { None });

        Ok(match entry_index {
            None => TranslationTableEntry::new_vacant(index, raw_entry, &mut self.table_descriptors),
            Some(descriptor_index) => TranslationTableEntry::new_occupied(index, raw_entry, &mut self.table_descriptors, descriptor_index),
        })
    }

    pub(super) fn allocate_region<F, R>(&mut self, first_block: usize, block_count: usize, attributes: EntryAttributes) -> VirtualMemoryResult<()>
    where F: FnMut(u64) -> R {
        Ok(())
    }
}

pub(super) struct DataBlock {
    data_block: DataBlockRaw,
}

impl DataBlock {
    pub(super) fn new(
        granule_size: GranuleSize,
        layer: u8,
    ) -> VirtualMemoryResult<Self> {
        let data_block = match (granule_size, layer) {
            //(GranuleSize::Kb4, 1) => 30, TODO: figure out how to support
            (GranuleSize::Kb4, 2) => DataBlockRaw::Mb2(Box::new(DataBlock2Mb::default())),
            (GranuleSize::Kb4, 3) => DataBlockRaw::Kb4(Box::new(DataBlock4Kb::default())),
            (GranuleSize::Kb16, 2) => DataBlockRaw::Mb32(Box::new(DataBlock32Mb::default())),
            (GranuleSize::Kb16, 3) => DataBlockRaw::Kb16(Box::new(DataBlock16Kb::default())),
            (GranuleSize::Kb64, 2) => DataBlockRaw::Mb512(Box::new(DataBlock512Mb::default())),
            (GranuleSize::Kb64, 3) => DataBlockRaw::Kb64(Box::new(DataBlock64Kb::default())),
            (_, _) => return Err(VirtualMemoryError::InvalidLevel),
        };

        Ok(DataBlock {
            data_block,
        })
    }
}

#[derive(Copy, Clone)]
pub(super) enum GranuleSize {
    Kb4 = 0b10,
    Kb16 = 0b01,
    Kb64 = 0b11,
}
