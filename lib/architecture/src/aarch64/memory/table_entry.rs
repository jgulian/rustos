use core::sync::atomic::{AtomicUsize, Ordering};
use alloc::sync::Arc;
use alloc::vec::Vec;
use crate::aarch64::memory::attributes::EntryAttributes;

use crate::aarch64::memory::translation_table::{DataBlock, TranslationTable};

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