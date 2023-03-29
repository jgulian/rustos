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



struct TranslationTableOccupiedEntry<'a>(usize, &'a mut u64, &'a mut Vec<(usize, TableDescriptor)>, usize);

struct TranslationTableVacantEntry<'a>(usize, &'a mut u64, &'a mut Vec<(usize, TableDescriptor)>);



pub(super) enum TableDescriptor {
    Block {
        data_block: Arc<DataBlock>,
        copy_on_write: Option<Arc<AtomicUsize>>,
        attributes: DescriptorAttributes,
    },
    Table {
        translation_table: Arc<TranslationTable>,
        copy_on_write: Option<Arc<AtomicUsize>>,
        attributes: DescriptorAttributes,
    },
    Transparent {
        address: usize,
        length: usize,
        attributes: DescriptorAttributes,
    },
}

impl TableDescriptor {
    fn value(&self) -> u64 {
        0
    }

    pub(super) fn clone(&mut self) -> Self {
        let copy_on_write = {
            let copy_on_write = match self {
                TableDescriptor::Block { copy_on_write, .. } => copy_on_write,
                TableDescriptor::Table { copy_on_write, .. } => copy_on_write,
                TableDescriptor::Transparent { address, length, attributes }
                => return TableDescriptor::Transparent { address: *address, length: *length, attributes: *attributes },
            };

            match copy_on_write {
                None => {
                    let copy_on_write_count = Arc::new(AtomicUsize::new(2));
                    *copy_on_write = Some(copy_on_write_count.clone());
                    copy_on_write_count
                }
                Some(copy_on_write_count) => {
                    copy_on_write_count.fetch_add(1, Ordering::SeqCst);
                    copy_on_write_count.clone()
                }
            }
        };

        match self {
            TableDescriptor::Block { data_block, attributes, .. } => TableDescriptor::Block {
                data_block: data_block.clone(),
                copy_on_write: Some(copy_on_write),
                attributes: *attributes,
            },
            TableDescriptor::Table {
                translation_table, attributes, ..
            } => TableDescriptor::Table {
                translation_table: translation_table.clone(),
                copy_on_write: Some(copy_on_write),
                attributes: *attributes,
            },
            _ => panic!("invalid control flow"),
        }
    }
}