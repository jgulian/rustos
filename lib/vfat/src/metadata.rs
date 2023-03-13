#[cfg(feature = "no_std")]
use alloc::boxed::Box;
#[cfg(not(feature = "no_std"))]
use std::boxed::Box;

use core::fmt;
use core::fmt::Formatter;

/// A date as represented in FAT32 on-disk structures.
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

/// A structure containing a date and time.
#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub struct Timestamp {
    time: Time,
    date: Date,
}

impl From<Date> for Timestamp {
    fn from(date: Date) -> Self {
        Timestamp {
            date,
            time: Time(0),
        }
    }
}

impl filesystem::filesystem::Timestamp for Timestamp {
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

impl fmt::Debug for Timestamp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use filesystem::filesystem::Timestamp;
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
#[derive(Default, Clone)]
pub struct Metadata {
    pub attributes: u8,
    pub created: Timestamp,
    pub last_access: Timestamp,
    pub last_modification: Timestamp,
}

impl filesystem::filesystem::Metadata for Metadata {
    fn read_only(&self) -> bool {
        self.attributes & (0b1) > 0
    }

    fn hidden(&self) -> bool {
        self.attributes & (0b10) > 0
    }

    fn created(&self) -> Box<dyn filesystem::filesystem::Timestamp> {
        Box::new(self.created)
    }

    fn accessed(&self) -> Box<dyn filesystem::filesystem::Timestamp> {
        Box::new(self.last_access)
    }

    fn modified(&self) -> Box<dyn filesystem::filesystem::Timestamp> {
        Box::new(self.last_modification)
    }
}

impl fmt::Debug for Metadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use filesystem::filesystem::Metadata;
        f.debug_struct("Metadata")
            .field("read_only", &self.read_only())
            .field("hidden", &self.hidden())
            //.field("created", self.created().borrow())
            //.field("accessed", self.accessed().borrow())
            //.field("modified", self.modified().borrow())
            .finish()
    }
}