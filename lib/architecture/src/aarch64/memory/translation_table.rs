use alloc::boxed::Box;
use alloc::sync::Arc;
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
    table_descriptors: Vec<(usize, TableDescriptor)>,
}

impl TranslationTable {
    pub(super) fn root(granule_size: GranuleSize) -> Self {
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
            granule_size,
            translation_level: 0,
            translation_table,
            table_descriptors: Vec::new(),
        }
    }

    pub(super) fn allocate_region<F>(&mut self, region_start: usize, region_size: usize, attributes: EntryAttributes, function: F) -> VirtualMemoryResult<()>
        where F: FnMut(u64, u64) -> TableDescriptor {
        self.allocate_region_offset(region_start, region_size, 0, attributes, function)
    }

    fn allocate_region_offset<F>(&mut self, region_start: usize, region_size: usize, offset: usize, attributes: EntryAttributes, function: F) -> VirtualMemoryResult<()>
        where F: FnMut(u64, u64) -> TableDescriptor {
        if !self.is_region_free(region_start, region_size)? {
            return Err(VirtualMemoryError::AlreadyAllocated);
        }

        let block_size = block_sizes_in_granule(self.granule_size)[self.translation_level] as usize;
        let first_index = region_start / block_size;
        let block_count = region_size / block_size;
        //TODO: make atomic
        (0..block_count).try_for_each(|i| {
            let index = first_index + i;
            let sub_region_start = if i == 0 { region_start % block_size } else { 0 };
            let sub_region_size = cmp::min(block_size - sub_region_start, region_size - block_size * i);

            let descriptor = match self.get(index) {
                None => {
                    return self.allocate_sub_region(index, sub_region_start, sub_region_size, block_size);
                }
                Some(descriptor) => descriptor,
            };

            match descriptor {
                TableDescriptor::Table { translation_table, .. } =>
                    translation_table.allocate_region_offset(first_sub_block, sub_region_size),
                _ => return Err(VirtualMemoryError::AlreadyAllocated),
            }

            Ok(())
        })
    }

    fn allocate_sub_region(&mut self, index: usize, sub_region_start: usize, sub_region_size: usize, block_size: usize) -> VirtualMemoryResult<()> {
        if sub_region_size == block_size {

        } else {

        }

        Ok(())
    }


    fn is_region_free(&self, region_start: usize, region_size: usize) -> VirtualMemoryResult<bool> {
        let minimum_block_size = minimum_block_size(self.granule_size) as usize;
        if region_start % minimum_block_size != 0 || region_size % minimum_block_size != 0 {
            return Err(VirtualMemoryError::NotPageAligned)
        }

        let block_size = block_sizes_in_granule(self.granule_size)[self.translation_level] as usize;
        let first_index = region_start / block_size;
        let block_count = region_size / block_size;
        (0..block_count).try_fold(true, |ok, i| {
            if !ok {
                return Ok(false);
            }

            let descriptor = match self.get(first_index + i) {
                None => return Ok(true),
                Some(descriptor) => descriptor
            };

            let sub_region_start = if i == 0 { region_start % block_size } else { 0 };
            let sub_region_size = cmp::min(block_size, region_size - block_size * i);

            match descriptor {
                TableDescriptor::Table { translation_table, .. } =>
                    translation_table.is_region_free(sub_region_start, sub_region_size),
                _ => return Ok(false),
            }
        })
    }

    fn get(&self, index: usize) -> Option<&TableDescriptor> {
        self.table_descriptors.iter()
            .find_map(|(i, table)| if *i == index { Some(table) } else { None })
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut TableDescriptor> {
        self.table_descriptors.iter_mut()
            .find_map(|(i, table)| if *i == index { Some(table) } else { None })
    }

    pub(super) fn len(granule_size: GranuleSize) -> usize {
        match granule_size {
            GranuleSize::Kb4 => TranslationTable4Kb::len(),
            GranuleSize::Kb16 => TranslationTable16Kb::len(),
            GranuleSize::Kb64 => TranslationTable64Kb::len(),
        }
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

fn block_sizes_in_granule(granule_size: GranuleSize) -> &'static [u64] {
    match granule_size {
        GranuleSize::Kb4 => &[0x40_000_000, 0x200_000, 0x1_000],
        GranuleSize::Kb16 => &[0x2_000_000, 0x4_000],
        GranuleSize::Kb64 => &[0x20_000_000, 0x10_000],
    }
}

fn minimum_block_size(granule_size: GranuleSize) -> u64 {
    match granule_size {
        GranuleSize::Kb4 => 0x1_000,
        GranuleSize::Kb16 => 0x4_000,
        GranuleSize::Kb64 => 0x10_000,
    }
}

fn get_alignment(mut address: u64) -> Option<usize> {
    let i = 0;

    while address != 0 {
        if address == 1 {
            return Some(i);
        } else {
            address >>= 1;
        }
    }

    None
}