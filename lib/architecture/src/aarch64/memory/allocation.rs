use core::ops::{Index, IndexMut};
use core::slice::SliceIndex;
use core::default::Default;

use alloc::boxed::Box;

macro_rules! block {
    ($name:ident, $size:literal, $T:ty) => {
        #[repr(align($size))]
        pub(super) struct $name([$T; Self::len()]);

        impl $name {
            pub(super) const fn len() -> usize {
                $size / core::mem::size_of::<$T>()
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self([0; Self::len()])
            }
        }

        impl<I> Index<I> for $name where I: SliceIndex<[$T], Output=$T> {
            type Output = $T;

            #[inline(always)]
            fn index(&self, index: I) -> &Self::Output {
                self.0.index(index)
            }
        }


        impl<I> IndexMut<I> for $name where I: SliceIndex<[$T], Output=$T> {
            #[inline(always)]
            fn index_mut(&mut self, index: I) -> &mut Self::Output {
                self.0.index_mut(index)
            }
        }
    };
}

block!(DataBlock4Kb, 0x1_000, u8);
block!(DataBlock16Kb, 0x4_000, u8);
block!(DataBlock64Kb, 0x10_000, u8);
block!(DataBlock2Mb, 0x200_000, u8);
block!(DataBlock32Mb, 0x2_000_000, u8);
block!(DataBlock512Mb, 0x20_000_000, u8);
//block!(Block1Gb, 0x40_000_000, u8);

block!(TranslationTable4Kb, 0x1_000, u64);
block!(TranslationTable16Kb, 0x4_000, u64);
block!(TranslationTable64Kb, 0x10_000, u64);

pub(super) enum DataBlockRaw {
    Kb4(Box<DataBlock4Kb>),
    Kb16(Box<DataBlock16Kb>),
    Kb64(Box<DataBlock64Kb>),
    Mb2(Box<DataBlock2Mb>),
    Mb32(Box<DataBlock32Mb>),
    Mb512(Box<DataBlock512Mb>),
}

pub(super) enum TranslationTableRaw {
    Kb4(Box<TranslationTable4Kb>),
    Kb16(Box<TranslationTable16Kb>),
    Kb64(Box<TranslationTable64Kb>),
}

fn
