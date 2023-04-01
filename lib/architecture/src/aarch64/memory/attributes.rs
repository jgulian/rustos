
macro_rules! get_bits {
    ($variable:expr, $bit:literal) => {
        ($variable >> $bit) & 1
    };
    ($variable:expr, $bit_beg:literal..$bit_end:literal) => {
        ($variable >> $bit_beg) & ((1 << ($bit_end - $bit_beg + 1)) - 1)
    };
}

macro_rules! set_bits {
    ($variable:expr, $bit:literal, $value:expr) => {
        $variable & !(1 << $bit) | ($value << $bit)
    };
    ($variable:expr, $bit_beg:literal..$bit_end:literal, $value:expr) => {
        $variable & !(((1 << ($bit_end - $bit_beg + 1)) - 1) << $bit_beg) | ($value << $bit_beg)
    };
}

#[derive(Copy, Clone)]
pub(super) struct EntryAttributes(u64);

impl EntryAttributes {
    pub(super) fn value(&self) -> u64 {
        self.0
    }

    pub(super) fn get_unprivileged_execute_never(&self) -> bool {
        get_bits!(self.0, 54) == 1
    }

    pub(super) fn unprivileged_execute_never(self, value: bool) -> Self {
        Self(set_bits!(self.0, 54, value as u64))
    }

    pub(super) fn get_address(&self) -> u64 {
        get_bits!(self.0, 16..47) << 16
    }

    pub(super) fn address(self, value: u64) -> Self {
        Self(set_bits!(self.0, 16..47, value >> 16))
    }

    pub(super) fn get_access_flag(&self) -> bool {
        get_bits!(self.0, 10) == 1
    }

    pub(super) fn access_flag(self, value: bool) -> Self {
        Self(set_bits!(self.0, 10, value as u64))
    }

    pub(super) fn get_shareability(&self) -> Shareability{
        Shareability::from(get_bits!(self.0, 8..9))
    }

    pub(super) fn shareability(self, value: Shareability) -> Self {
        Self(set_bits!(self.0, 8..9, value as u64))
    }

    pub(super) fn get_access_permissions(&self) -> AccessPermissions {
        AccessPermissions::from(get_bits!(self.0, 6..7))
    }

    pub(super) fn access_permissions(self, value: AccessPermissions) -> Self {
        Self(set_bits!(self.0, 6..7, value as u64))
    }

    pub(super) fn get_not_sharable(&self) -> bool {
        get_bits!(self.0, 5) == 1
    }

    pub(super) fn not_sharable(self, value: bool) -> Self {
        Self(set_bits!(self.0, 5, value as u64))
    }

    pub(super) fn get_mair_index(&self) -> u64 {
        get_bits!(self.0, 2..4)
    }

    pub(super) fn mair_index(self, value: u64) -> Self {
        Self(set_bits!(self.0, 2..4, value))
    }

    pub(super) fn block(self) -> Self {
        Self(set_bits!(self.0, 1, 0))
    }

    pub(super) fn is_block(self) -> bool {
        get_bits!(self.0, 1) == 0
    }

    pub(super) fn table(self) -> Self {
        Self(set_bits!(self.0, 1, 1))
    }

    pub(super) fn is_table(self) -> bool {
        get_bits!(self.0, 1) == 1
    }

    pub(super) fn is_valid(self) -> bool {
        get_bits!(self.0, 0) == 0
    }

    pub(super) fn invalidate(self) -> Self {
        Self(set_bits!(self.0, 0, 0))
    }

    pub(super) fn validate(self) -> Self {
        Self(set_bits!(self.0, 0, 1))
    }
}

impl Default for EntryAttributes {
    fn default() -> Self {
        Self(0)
    }
}

impl Into<u64> for EntryAttributes {
    fn into(self) -> u64 {
        self.0
    }
}

#[derive(Copy, Clone)]
pub(super) enum Shareability {
    OuterSharable = 0b10,
    InnerSharable = 0b11,
}

impl From<u64> for Shareability {
    fn from(value: u64) -> Self {
        match value {
            0b10 => Shareability::OuterSharable,
            0b11 => Shareability::InnerSharable,
            _ => panic!("unsupported shareability"),
        }
    }
}

#[derive(Copy, Clone)]
pub(super) enum StageAttributes {
    One {
        // Bit 8
        // not_global: bool,
        /// Bit 7
        other_exception_access: OtherExceptionAccess,
        /// Bit 6
        lower_exception_access: bool,
        // Bit 5
        // non_secure: bool,
        mair_index: u8,
    },
    Two {
        /// Bits 7:6
        access_permissions: AccessPermissions,
        /// Bits 5:2
        memory_region_type: MemoryRegionType,
    },
}

#[derive(Copy, Clone)]
pub(super) enum OtherExceptionAccess {
    ReadWrite = 0b0,
    ReadOnly = 0b1,
}

#[derive(Copy, Clone)]
pub(super) enum AccessPermissions {
    None = 0b00,
    ReadOnly = 0b01,
    WriteOnly = 0b10,
    ReadWrite = 0b11,
}

impl From<u64> for AccessPermissions {
    fn from(value: u64) -> Self {
        match value {
            0b00 => AccessPermissions::None,
            0b01 => AccessPermissions::ReadOnly,
            0b10 => AccessPermissions::WriteOnly,
            0b11 => AccessPermissions::ReadWrite,
            _ => panic!("unsupported shareability")
        }
    }
}

#[derive(Copy, Clone)]
pub(super) enum MemoryRegionType {
    Device(DeviceMemory),
    NormalNonCacheable(InnerCacheable),
    NormalWriteThrough(InnerCacheable),
    NormalWriteBack(InnerCacheable),
}

impl Into<u64> for MemoryRegionType {
    fn into(self) -> u64 {
        match self {
            MemoryRegionType::Device(device_memory) => device_memory as u64,
            MemoryRegionType::NormalNonCacheable(inner_cacheable) => {
                0b0100 | inner_cacheable as u64
            }
            MemoryRegionType::NormalWriteThrough(inner_cacheable) => {
                0b1000 | inner_cacheable as u64
            }
            MemoryRegionType::NormalWriteBack(inner_cacheable) => 0b1100 | inner_cacheable as u64,
        }
    }
}

#[derive(Copy, Clone)]
pub(super) enum DeviceMemory {
    NoGatheringNoReorderingNoEarlyWriteAcknowledgement = 0b00,
    NoGatheringNoReorderingEarlyWriteAcknowledgement = 0b01,
    NoGatheringReorderingEarlyWriteAcknowledgement = 0b10,
    GatheringReorderingEarlyWriteAcknowledgement = 0b11,
}

#[derive(Copy, Clone)]
pub(super) enum InnerCacheable {
    NonCacheable = 0b01,
    WriteThroughCacheable = 0b10,
    WriteBackCacheable = 0b11,
}