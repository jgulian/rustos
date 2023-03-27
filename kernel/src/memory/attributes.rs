pub(super) struct DescriptorAttributes {
    // stage: u8, TODO: needed?
    /// Software Defined
    /// Bit 55
    copy_on_write: bool,

    /// Bit 54
    unprivileged_execute_never: bool,
    // Bit 53
    // privileged_execute_never: bool, TODO: support
    // continuous: bool,
    // dirty_bit_modifier: bool,
    access_flag: bool,
    sharable: Shareability,
    stage_attributes: StageAttributes,
}

pub(super) enum Shareability {}

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

pub(super) enum OtherExceptionAccess {
    ReadWrite = 0b0,
    ReadOnly = 0b1,
}


pub(super) enum AccessPermissions {
    None = 0b00,
    ReadOnly = 0b01,
    WriteOnly = 0b10,
    ReadWrite = 0b11,
}

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
            MemoryRegionType::NormalNonCacheable(inner_cacheable) =>
                0b0100 | inner_cacheable as u64,
            MemoryRegionType::NormalWriteThrough(inner_cacheable) =>
                0b1000 | inner_cacheable as u64,
            MemoryRegionType::NormalWriteBack(inner_cacheable) =>
                0b1100 | inner_cacheable as u64,
        }
    }
}

pub(super) enum DeviceMemory {
    NoGatheringNoReorderingNoEarlyWriteAcknowledgement = 0b00,
    NoGatheringNoReorderingEarlyWriteAcknowledgement = 0b01,
    NoGatheringReorderingEarlyWriteAcknowledgement = 0b10,
    GatheringReorderingEarlyWriteAcknowledgement = 0b11,
}

pub(super) enum InnerCacheable {
    NonCacheable = 0b01,
    WriteThroughCacheable = 0b10,
    WriteBackCacheable = 0b11,
}