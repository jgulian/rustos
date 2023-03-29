use core::arch::asm;
use core::sync::atomic::Ordering;
use aarch64::{ID_AA64MMFR0_EL1, isb, MAIR_EL1, TCR_EL1};
use crate::aarch64::memory::attributes::DescriptorAttributes;
use crate::aarch64::memory::page_table::{GranuleSize, TableDescriptor, TranslationTable};
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

        let level_0_translation_table = TranslationTable::new(GRANULE_SIZE);

        let mut level_1_translation_table = TranslationTable::new(GRANULE_SIZE);
        for i in 0..level_1_translation_table.len() {
            let entry = level_1_translation_table.entry(i).unwrap();
            entry.update(TableDescriptor::Transparent {
                address: 0,
                length: 0,
                attributes: DescriptorAttributes {},
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