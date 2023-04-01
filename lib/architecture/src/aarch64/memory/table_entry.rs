use core::sync::atomic::{AtomicUsize, Ordering};
use alloc::sync::Arc;
use alloc::vec::Vec;
use crate::aarch64::memory::attributes::EntryAttributes;

use crate::aarch64::memory::translation_table::{DataBlock, TranslationTable};

pub(super) enum TranslationTableEntry<'a> {
    Occupied(TranslationTableOccupiedEntry<'a>),
    Vacant(TranslationTableVacantEntry<'a>),
}

impl<'a> TranslationTableEntry<'a> {
    pub(super) fn new_occupied(index: usize, raw_entry: &mut u64, table_descriptors: &mut Vec<(usize, TableDescriptor)>, descriptor_index: usize) -> Self {
        Self::Occupied(TranslationTableOccupiedEntry(
            index,
            raw_entry,
            table_descriptors,
            descriptor_index,
        ))
    }

    pub(super) fn new_vacant(index: usize, raw_entry: &mut u64, table_descriptors: &mut Vec<(usize, TableDescriptor)>) -> Self {
        Self::Vacant(TranslationTableVacantEntry(
            index,
            raw_entry,
            table_descriptors,
        ))
    }

    pub(super) fn update(self, attributes: EntryAttributes, table_descriptor: TableDescriptor) -> Self {
        let attributes_with_address = table_descriptor.update_attributes(attributes);
        match self {
            TranslationTableEntry::Occupied(occupied_entry) => {
                *occupied_entry.1 = attributes_with_address.value();
                occupied_entry.2[occupied_entry.3] = (occupied_entry.0, table_descriptor);
                TranslationTableEntry::Occupied(occupied_entry)
            }
            TranslationTableEntry::Vacant(vacant_entry) => {
                *vacant_entry.1 = attributes_with_address.value();
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

struct TranslationTableOccupiedEntry<'a>(usize, &'a mut u64, &'a mut Vec<(usize, TableDescriptor)>, usize);

struct TranslationTableVacantEntry<'a>(usize, &'a mut u64, &'a mut Vec<(usize, TableDescriptor)>);


pub(super) enum TableDescriptor {
    Block {
        data_block: Arc<DataBlock>,
        copy_on_write: Option<Arc<AtomicUsize>>,
    },
    Table {
        translation_table: Arc<TranslationTable>,
        copy_on_write: Option<Arc<AtomicUsize>>,
    },
    Transparent {
        address: u64,
        length: u64,
    },
}

impl TableDescriptor {
    pub(super) fn update_attributes(&self, attributes: EntryAttributes) -> EntryAttributes {
        match self {
            TableDescriptor::Block { data_block, .. } =>
                attributes.address(Arc::as_ptr(data_block) as u64),
            TableDescriptor::Table { translation_table, .. } =>
            attributes.address(Arc::as_ptr(translation_table) as u64),
            TableDescriptor::Transparent { address, .. } =>
                attributes.address(*address),
        }
    }

    pub(super) fn clone(&mut self) -> Self {
        let copy_on_write = {
            let copy_on_write = match self {
                TableDescriptor::Block { copy_on_write, .. } => copy_on_write,
                TableDescriptor::Table { copy_on_write, .. } => copy_on_write,
                TableDescriptor::Transparent { address, length }
                => return TableDescriptor::Transparent { address: *address, length: *length },
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
            TableDescriptor::Block { data_block, .. } => TableDescriptor::Block {
                data_block: data_block.clone(),
                copy_on_write: Some(copy_on_write),
            },
            TableDescriptor::Table {
                translation_table, ..
            } => TableDescriptor::Table {
                translation_table: translation_table.clone(),
                copy_on_write: Some(copy_on_write),
            },
            _ => panic!("invalid control flow"),
        }
    }
}