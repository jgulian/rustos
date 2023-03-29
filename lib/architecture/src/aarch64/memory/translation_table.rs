use alloc::boxed::Box;
use crate::aarch64::memory::allocation::{
    DataBlock16Kb, DataBlock2Mb, DataBlock32Mb, DataBlock4Kb, DataBlock512Mb, DataBlock64Kb,
    DataBlockRaw, TranslationTable16Kb, TranslationTable4Kb, TranslationTable64Kb,
    TranslationTableRaw,
};
use crate::aarch64::memory::attributes::DescriptorAttributes;
use crate::primitives::memory::{VirtualMemoryError, VirtualMemoryResult};

use core::sync::atomic::{AtomicUsize, Ordering};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::ops::{Index, IndexMut};

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
            None => TranslationTableEntry::Vacant(TranslationTableVacantEntry(index, raw_entry, &mut self.table_descriptors)),
            Some(index) => TranslationTableEntry::Occupied(TranslationTableOccupiedEntry(index, raw_entry, &mut self.table_descriptors, index)),
        })
    }
}

pub(super) enum TranslationTableEntry<'a> {
    Occupied(TranslationTableOccupiedEntry<'a>),
    Vacant(TranslationTableVacantEntry<'a>),
}

impl<'a> TranslationTableEntry<'a> {
    pub(super) fn update(self, table_descriptor: TableDescriptor) -> Self {
        match self {
            TranslationTableEntry::Occupied(occupied_entry) => {
                *occupied_entry.1 = table_descriptor.value();
                occupied_entry.2[occupied_entry.3] = (occupied_entry.0, table_descriptor);
                TranslationTableEntry::Occupied(occupied_entry)
            }
            TranslationTableEntry::Vacant(vacant_entry) => {
                *vacant_entry.1 = table_descriptor.value();
                let descriptor_index = vacant_entry.2.len();
                vacant_entry.2.push((vacant_entry.0, table_descriptor));
                TranslationTableEntry::Occupied(TranslationTableOccupiedEntry(
                    vacant_entry.0,
                    vacant_entry.1,
                    vacant_entry.2,
                    descriptor_index,
                ))
            }
        }
    }
}

struct DataBlock {
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
