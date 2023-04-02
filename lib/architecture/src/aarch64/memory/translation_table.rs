use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::collections::BTreeMap;
use crate::aarch64::memory::allocation::{
    DataBlock16Kb, DataBlock2Mb, DataBlock32Mb, DataBlock4Kb, DataBlock512Mb, DataBlock64Kb,
    DataBlockRaw, TranslationTable16Kb, TranslationTable4Kb, TranslationTable64Kb,
    TranslationTableRaw,
};
use crate::primitives::memory::{VirtualMemoryError, VirtualMemoryResult};

use alloc::vec::Vec;
use core::cmp;

use crate::aarch64::memory::attributes::EntryAttributes;
use crate::aarch64::memory::table_entry::TableDescriptor;

pub(super) struct TranslationTable {
    granule_size: GranuleSize,
    translation_level: usize,
    translation_table: TranslationTableRaw,
    table_descriptors: BTreeMap<usize, TableDescriptor>,
}

impl TranslationTable {
    pub(super) fn root(granule_size: GranuleSize) -> Self {
        Self::new(granule_size, 0).unwrap()
    }

    fn new(granule_size: GranuleSize, translation_level: usize) -> Option<Self> {
        let translation_table = match granule_size {
            GranuleSize::Kb4 => {
                if translation_level > 3 {
                    return None;
                }
                TranslationTableRaw::Kb4(Box::new(TranslationTable4Kb::default()))
            },
            GranuleSize::Kb16 => {
                //TODO: ok so granule kb16 won't work for level 0 pages
                if translation_level > 2 {
                    return None;
                }
                TranslationTableRaw::Kb16(Box::new(TranslationTable16Kb::default()))
            }
            GranuleSize::Kb64 => {
                if translation_level > 2 {
                    return None;
                }
                TranslationTableRaw::Kb64(Box::new(TranslationTable64Kb::default()))
            }
        };

        Some(
            TranslationTable {
                granule_size,
                translation_level,
                translation_table,
                table_descriptors: BTreeMap::new(),
            }
        )
    }

    pub(super) fn allocate_region<F>(&mut self, region_start: usize, region_size: usize, attributes: EntryAttributes, function: F) -> VirtualMemoryResult<()>
        where F: FnMut(usize, usize) -> (TableDescriptor, EntryAttributes) {
        self.allocate_region_offset(region_start, region_start + region_size, 0, function)
    }

    fn allocate_region_offset<F>(&mut self, region_start: usize, region_end: usize, offset: usize, function: F) -> VirtualMemoryResult<()>
        where F: FnMut(usize, usize) -> (TableDescriptor, EntryAttributes) {
        let block_size = *block_sizes_in_granule(self.granule_size).get(self.translation_level)
            .ok_or(VirtualMemoryError::NotPageAligned)? as usize;
        if !self.is_region_free(region_start, region_end)? {
            return Err(VirtualMemoryError::AlreadyAllocated);
        }

        //TODO: make it actually atomic?
        self.get_sub_regions(region_start, region_end)?
            .filter(|(_, _, _, has_region_started, has_region_ended)| *has_region_started || *has_region_ended)
            .try_for_each(|(i, sub_region_start, sub_region_end, _, _)| {
                let current_offset = offset + i * block_size;
                match self.table_descriptors.get_mut(&i) {
                    None => {
                        if sub_region_end - sub_region_start % block_size == 0 {
                            let (table_descriptor, mut attributes) = function(current_offset, data_block.data_block.as_mut_slice());
                            attributes = table_descriptor.update_attributes(attributes);
                            self.table_descriptors.insert(i, table_descriptor);
                            self.translation_table[i] = attributes.value();
                        } else {
                            let new_translation_table = TranslationTable::new(self.granule_size, self.translation_level + 1);
                        }
                        Ok(())
                    }
                    Some(table_descriptor) => {
                        match table_descriptor {
                            TableDescriptor::Table { translation_table, .. } =>
                                translation_table.allocate_region_offset(sub_region_start, sub_region_end, current_offset, function),
                            _ => Err(VirtualMemoryError::AlreadyAllocated)
                        }
                    }
                }
            })
    }

    fn is_region_free(&self, region_start: usize, region_end: usize) -> VirtualMemoryResult<bool> {
        self.get_sub_regions(region_start, region_end)?
            .filter(|(_, _, _, has_region_started, has_region_ended)| *has_region_started || *has_region_ended)
            .try_fold(true, |ok, (i, sub_region_start, sub_region_end, _, _)| {
                if !ok {
                    return Ok(false);
                }

                match self.table_descriptors.get(&i) {
                    None => return Ok(true),
                    Some(table_descriptor) => {
                        match table_descriptor {
                            TableDescriptor::Table { translation_table, .. } =>
                                translation_table.is_region_free(sub_region_start, sub_region_end),
                            _ => Ok(false)
                        }
                    }
                }
            })
    }

    fn get_sub_regions(&self, region_start: usize, region_end: usize) -> VirtualMemoryResult<impl Iterator<Item=(usize, usize, usize, bool, bool)>> {
        let block_size = *block_sizes_in_granule(self.granule_size).get(self.translation_level)
            .ok_or(VirtualMemoryError::NotPageAligned)? as usize;

        Ok(
            (0..translation_table_size(self.granule_size))
                .map(move |i| {
                    let mut sub_region_start = i * block_size;
                    let mut sub_region_end = (i + 1) * block_size;
                    let has_region_started = region_start <= sub_region_start;
                    if has_region_started {
                        sub_region_start = region_start % block_size;
                    }
                    let has_region_ended = sub_region_end <= region_end;
                    if has_region_ended {
                        sub_region_end = sub_region_end % block_size;
                    }

                    (i, sub_region_start, sub_region_end, has_region_started, has_region_ended)
                })
        )
    }

    fn get(&self, index: usize) -> Option<&TableDescriptor> {
        self.table_descriptors.iter()
            .find_map(|(i, table)| if *i == index { Some(table) } else { None })
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut TableDescriptor> {
        self.table_descriptors.iter_mut()
            .find_map(|(i, table)| if *i == index { Some(table) } else { None })
    }
}

pub(super) struct DataBlock {
    data_block: DataBlockRaw,
}

impl DataBlock {
    fn new(
        granule_size: GranuleSize,
        level: usize,
    ) -> VirtualMemoryResult<Self> {
        let data_block = match (granule_size, level) {
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

fn block_sizes_in_granule(granule_size: GranuleSize) -> &'static [u64] {
    match granule_size {
        GranuleSize::Kb4 => &[0x8_000_000_000, 0x40_000_000, 0x200_000, 0x1_000],
        GranuleSize::Kb16 => &[0x1_000_000_000, 0x2_000_000, 0x4_000],
        GranuleSize::Kb64 => &[0x40_000_000_000, 0x20_000_000, 0x10_000],
    }
}

fn minimum_block_size(granule_size: GranuleSize) -> u64 {
    match granule_size {
        GranuleSize::Kb4 => 0x1_000,
        GranuleSize::Kb16 => 0x4_000,
        GranuleSize::Kb64 => 0x10_000,
    }
}

pub(super) fn translation_table_size(granule_size: GranuleSize) -> usize {
    match granule_size {
        GranuleSize::Kb4 => TranslationTable4Kb::len(),
        GranuleSize::Kb16 => TranslationTable16Kb::len(),
        GranuleSize::Kb64 => TranslationTable64Kb::len(),
    }
}