use core::arch::asm;
use core::cmp::min;
use core::sync::atomic::Ordering;
use aarch64::{ID_AA64MMFR0_EL1, isb, MAIR_EL1, TCR_EL1};
use crate::aarch64::memory::attributes::{AccessPermissions, DescriptorAttributes, EntryAttributes, Shareability};
use crate::aarch64::memory::page_table::{GranuleSize, TableDescriptor, TranslationTable};
use crate::aarch64::memory::table_entry::TableDescriptor;
use crate::aarch64::memory::translation_table::TranslationTable;
use crate::primitives;

const GRANULE_SIZE: GranuleSize = GranuleSize::Kb4;

struct Aarch64Memory {
    kernel_translation_table: TranslationTable,
    granule_size: GranuleSize,
}

impl primitives::MemoryManager for Aarch64Memory {
    type MemorySegmentIterator = ();
    type ProcessMemory = Aarch64ProcessMemory;

    fn new() -> Self
        where
            Self: Sized,
    {
        //assert_eq!(unsafe { ID_AA64MMFR0_EL1.get_value(ID_AA64MMFR0_EL1::TGran64 }), 0);

        let ips = unsafe { ID_AA64MMFR0_EL1.get_value(ID_AA64MMFR0_EL1::PARange) };


        unsafe {
            // (ref. D7.2.70: Memory Attribute Indirection Register)
            MAIR_EL1.set(
                (0xFF << 00) | // AttrIdx=0: normal, IWBWA, OWBWA, NTR
                    (0x04 << 08) | // AttrIdx=1: device, nGnRE (must be OSH too)
                    (0x44 << 16), // AttrIdx=2: non cacheable
            );
        }

        // (ref. D7.2.91: Translation Control Register)
        unsafe {
            TCR_EL1.set(
                (ips << 32) | // IPS
                    ((GRANULE_SIZE as u64) << 30) | // TG1=64k
                    (0b11 << 28) | // SH1=3 inner
                    (0b01 << 26) | // ORGN1=1 write back
                    (0b01 << 24) | // EPD1 enables higher half
                    ((USER_MASK_BITS as u64) << 16) | // T1SZ=34 (1GB)
                    (0b01 << 14) | // TG0=64k
                    (0b11 << 12) | // SH0=3 inner
                    (0b01 << 10) | // ORGN0=1 write back
                    (0b01 << 8) | // EPD0 enables lower half
                    (KERNEL_MASK_BITS as u64), // T0SZ=31 (8GB)
            );
        }
        isb();

        let kernel_memory_table_pages = EntryAttributes::default()
            .validate()
            .table()
            .access_flag(true)
            .mair_index(0)
            .access_permissions(AccessPermissions::ReadWrite)
            .shareability(Shareability::InnerSharable);
        let kernel_memory_data_pages = kernel_memory_table_pages.block();

        let level_0_translation_table = TranslationTable::new(GRANULE_SIZE);

        let granule_table_len = level_0_translation_table.len();
        let memory_limit = 0x3c_000_000;
        let level_two_page_size = 0x200_000;
        let level_2_translation_table_count = memory_limit / level_two_page_size;
        let level_1_translation_table_count: usize = level_2_translation_table_count / granule_table_len;

        for i in 0..level_1_translation_table_count {
            let is_last = i == level_1_translation_table_count - 1;
            let specific_level_1_count = if !is_last { granule_table_len } else { level_2_translation_table_count % granule_table_len };
            let mut level_1_translation_table = TranslationTable::new(GRANULE_SIZE);

            for j in 0..specific_level_1_count {
                let entry = level_1_translation_table.entry(j)
                    .expect("unable to get entry from level 1 table");
                let location = (i * granule_table_len + j) * level_two_page_size;
                entry.update(kernel_memory_data_pages, TableDescriptor::Transparent {
                    address: location as u64,
                    length: level_two_page_size as u64,
                });
            }


        }


        let mut level_1_translation_table = TranslationTable::new(GRANULE_SIZE);
        let level_2_translation_table_count

        (0..)
        .skip(0x200_000)


        for i in 0..level_1_translation_table.len() {
            let entry = level_1_translation_table.entry(i).unwrap();
            entry.update(kernel_memory_pages, TableDescriptor::Transparent {
                address: (0x40_000_000 * i) as u64,
                length: 0x40_000_000,
            });
        }


        let baddr = self.kern_pt_addr.load(Ordering::Relaxed);

        TTBR0_EL1.set(baddr as u64);
        TTBR1_EL1.set(baddr as u64);

        asm!("dsb ish");
        isb();

        SCTLR_EL1.set(SCTLR_EL1.get() | SCTLR_EL1::I | SCTLR_EL1::C | SCTLR_EL1::M);
        asm!("dsb sy");
        isb();

        set_mmu_ready();

        Self {
            granule_size: GranuleSize::Kb4,
        }
    }

    fn memory_map() -> Self::MemorySegmentIterator {
        todo!()
    }

    fn create_process_memory(&mut self) -> Self::ProcessMemory {
        Aarch64ProcessMemory {
            granule_size: self.granule_size,
        }
    }
}

struct Aarch64ProcessMemory {
    granule_size: GranuleSize,
    root_table_descriptor: TableDescriptor,
}

impl primitives::ProcessMemory for Aarch64ProcessMemory {
    type AllocationSizeIterator = ();

    fn allocate(&mut self, virtual_address: usize, block_size: usize) {
        todo!()
    }

    fn deallocate(&mut self, virtual_address: usize) {
        todo!()
    }

    fn translate(&mut self, virtual_address: usize) -> &mut [u8] {
        todo!()
    }

    fn clone(&mut self) -> Self where Self: Sized {}

    fn block_sizes(&self) -> Self::AllocationSizeIterator {
        []
    }
}

impl Clone for Aarch64ProcessMemory {
    fn clone(&self) -> Self {
        todo!()
    }
}