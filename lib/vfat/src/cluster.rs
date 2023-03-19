#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Copy, Clone, Hash)]
pub(crate) struct Cluster(u32);

impl From<u32> for Cluster {
    fn from(raw_num: u32) -> Cluster {
        Cluster(raw_num & !(0xF << 28))
    }
}

impl Cluster {
    pub(crate) fn fat_sector_number(&self, reserved_sector_count: u64, bytes_per_sector: u16) -> u64 {
        reserved_sector_count + (self.0 as u64 * 4 / bytes_per_sector as u64)
    }

    pub(crate) fn fat_entry_offset(&self, bytes_per_sector: u16) -> u32 {
        (self.0 * 4) % bytes_per_sector as u32
    }

    pub(crate) fn sector_start(&self, data_start_sector: u64, sectors_per_cluster: u8) -> u64 {
        data_start_sector + (self.0 as u64 - 2) * sectors_per_cluster as u64
    }

    pub(crate) fn offset(&self) -> u32 {
        self.0 * 4
    }

    pub(crate) fn high_and_low(&self) -> (u16, u16) {
        ((self.0 >> 16) as u16, self.0 as u16)
    }
}

impl Into<u32> for Cluster {
    fn into(self) -> u32 {
        self.0
    }
}

impl Into<u64> for Cluster {
    fn into(self) -> u64 {
        self.0 as u64
    }
}