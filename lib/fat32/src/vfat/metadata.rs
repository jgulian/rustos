use core::fmt;

use core::fmt::Formatter;
use filesystem;

/// A date as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Date(u16);

impl Date {
    fn year(&self) -> usize {
        ((self.0 as usize >> 9) & 0b111_1111_usize) + 1980
    }

    fn month(&self) -> u8 {
        ((self.0 >> 5) & 0b1111_u16) as u8
    }

    fn day(&self) -> u8 {
        (self.0 & 0b11111_u16) as u8
    }
}

/// Time as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Time(u16);

impl Time {
    fn hour(&self) -> u8 {
        ((self.0 >> 11) & 0b11111_u16) as u8
    }

    fn minute(&self) -> u8 {
        ((self.0 >> 5) & 0b111111_u16) as u8
    }

    fn second(&self) -> u8 {
        ((self.0 & 0b11111_u16) << 1) as u8
    }
}

/// File attributes as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Attributes(u8);

/// A structure containing a date and time.
#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
pub struct Timestamp {
    pub time: Time,
    pub date: Date,
}

impl From<Date> for Timestamp {
    fn from(date: Date) -> Self {
        Timestamp{date, time: Time(0)
        }
    }
}

impl filesystem::Timestamp for Timestamp {
    fn year(&self) -> usize {
        self.date.year()
    }

    fn month(&self) -> u8 {
        self.date.month()
    }

    fn day(&self) -> u8 {
        self.date.day()
    }

    fn hour(&self) -> u8 {
        self.time.hour()
    }

    fn minute(&self) -> u8 {
        self.time.minute()
    }

    fn second(&self) -> u8 {
        self.time.second()
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use filesystem::Timestamp;
        f.debug_struct("Timestamp")
            .field("year", &self.year())
            .field("month", &self.month())
            .field("day", &self.day())
            .field("hour", &self.hour())
            .field("minute", &self.minute())
            .field("second", &self.second())
            .finish()
    }
}

/// Metadata for a directory entry.
#[derive(Default, Debug, Clone)]
pub struct Metadata {
    pub attributes: u8,
    pub created: Timestamp,
    pub last_access: Timestamp,
    pub last_modification: Timestamp,
}

impl filesystem::Metadata for Metadata {
    type Timestamp = Timestamp;

    fn read_only(&self) -> bool {
        self.attributes & (0b1) > 0
    }

    fn hidden(&self) -> bool {
        self.attributes & (0b10) > 0
    }

    fn created(&self) -> Self::Timestamp {
        self.created
    }

    fn accessed(&self) -> Self::Timestamp {
        self.last_access
    }

    fn modified(&self) -> Self::Timestamp {
        self.last_modification
    }
}

impl fmt::Display for Metadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use filesystem::Metadata;
        f.debug_struct("Metadata")
            .field("read_only", &self.read_only())
            .field("hidden", &self.hidden())
            .field("created", &self.created())
            .field("accessed", &self.accessed())
            .field("modified", &self.modified())
            .finish()
    }
}
