use alloc::boxed::Box;
use alloc::vec::Vec;
use core::alloc::{GlobalAlloc, Layout};
use crate::ALLOCATOR;
use crate::memory::attributes::DescriptorAttributes;
use crate::memory::error::{VirtualMemoryError, VirtualMemoryResult};

pub(super) struct TranslationTable {
    table_address: *mut u64,
    layout: Layout,
    table_descriptors: Vec<TableDescriptor>,
    attributes: DescriptorAttributes,
}

impl TranslationTable {
    fn new(granule_size: GranuleSize, attributes: DescriptorAttributes) -> VirtualMemoryResult<Self> {
        let table_size_bits: usize = match granule_size {
            GranuleSize::Kb4 => 12,
            GranuleSize::Kb16 => 14,
            GranuleSize::Kb64 => 16,
        };
        let table_size = 1usize << table_size_bits;
        let layout = Layout::from_size_align(table_size, table_size)?;

        let table_address = unsafe { ALLOCATOR.alloc(layout) as *mut u64 };
        if table_address as usize == 0 {
            return Err(VirtualMemoryError::AllocationFailed);
        }

        Ok(Block {
            table_address,
            layout,
            attributes,
        })
    }
}

struct Block {
    page_address: *mut u8,
    layout: Layout,
    attributes: DescriptorAttributes,
}

impl Block {
    fn new(granule_size: GranuleSize, layer: u8, attributes: DescriptorAttributes) -> VirtualMemoryResult<Self> {
        let page_size_bits: usize = match (granule_size, layer) {
            (GranuleSize::Kb4, 1) => 30,
            (GranuleSize::Kb4, 2) => 21,
            (GranuleSize::Kb4, 3) => 12,
            (GranuleSize::Kb16, 2) => 25,
            (GranuleSize::Kb16, 3) => 14,
            (GranuleSize::Kb64, 2) => 29,
            (GranuleSize::Kb64, 3) => 16,
            (_, _) => return Err(VirtualMemoryError::InvalidLevel),
        };
        let page_size = 1usize << page_size_bits;
        let layout = Layout::from_size_align(page_size, page_size)?;

        let page_address = unsafe { ALLOCATOR.alloc(layout) };
        if page_address as usize == 0 {
            return Err(VirtualMemoryError::AllocationFailed);
        }

        Ok(Block {
            page_address,
            layout,
            attributes,
        })
    }
}

impl Drop for Block {
    fn drop(&mut self) {
        unsafe {
            ALLOCATOR.dealloc(self.page, self.layout)
        }
    }
}

enum TableDescriptor {
    Block(Block),
    Table(TranslationTable),
}

enum GranuleSize {
    Kb4 = 0b10,
    Kb16 = 0b01,
    Kb64 = 0b11,
}