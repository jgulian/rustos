use alloc::string::String;
use shim::{io, path::Path};
use shim::path::PathBuf;

use crate::Metadata;

/// Trait implemented by files in the file system.
pub trait File: io::Read + io::Write + io::Seek + Sized {
    /// Writes any buffered data to disk.
    fn sync(&mut self) -> io::Result<()>;

    /// Returns the size of the file in bytes.
    fn size(&self) -> u64;
}

/// Trait implemented by directories in a file system.
pub trait Dir: Sized {
    /// The type of entry stored in this directory.
    type Entry: Entry;

    /// An type that is an iterator over the entries in this directory.
    type Iter: Iterator<Item = Self::Entry>;

    /// Returns an interator over the entries in this directory.
    fn entries(&mut self) -> io::Result<Self::Iter>;

    fn append(&mut self, entry: Self::Entry) -> io::Result<()>;

    fn remove(&mut self, entry: Self::Entry) -> io::Result<()>;
}

/// Trait implemented by directory entries in a file system.
///
/// An entry is either a `File` or a `Directory` and is associated with both
/// `Metadata` and a name.
pub trait Entry: Sized {
    type File: File;
    type Dir: Dir;
    type Metadata: Metadata;

    /// The name of the file or directory corresponding to this entry.
    fn name(&self) -> &str;

    /// The metadata associated with the entry.
    fn metadata(&self) -> &Self::Metadata;

    /// If `self` is a file, returns `Some` of a reference to the file.
    /// Otherwise returns `None`.
    fn as_file(&self) -> Option<&Self::File>;

    /// If `self` is a directory, returns `Some` of a reference to the
    /// directory. Otherwise returns `None`.
    fn as_dir(&self) -> Option<&Self::Dir>;

    /// If `self` is a file, returns `Some` of the file. Otherwise returns
    /// `None`.
    fn into_file(self) -> Option<Self::File>;

    /// If `self` is a directory, returns `Some` of the directory. Otherwise
    /// returns `None`.
    fn into_dir(self) -> Option<Self::Dir>;

    /// Returns `true` if this entry is a file or `false` otherwise.
    fn is_file(&self) -> bool {
        self.as_file().is_some()
    }

    /// Returns `true` if this entry is a directory or `false` otherwise.
    fn is_dir(&self) -> bool {
        self.as_dir().is_some()
    }
}

/// Trait implemented by file systems.
pub trait FileSystem: Sized {
    /// The type of files in this file system.
    type File: File;

    /// The type of directories in this file system.
    type Dir: Dir<Entry = Self::Entry>;

    /// The type of directory entries in this file system.
    type Entry: Entry<File = Self::File, Dir = Self::Dir>;

    /// Opens the entry at `path`. `path` must be absolute.
    ///
    /// # Errors
    ///
    /// If `path` is not absolute, an error kind of `InvalidInput` is returned.
    ///
    /// If any component but the last in `path` does not refer to an existing
    /// directory, an error kind of `InvalidInput` is returned.
    ///
    /// If there is no entry at `path`, an error kind of `NotFound` is returned.
    ///
    /// All other error values are implementation defined.
    fn open(&mut self, path: &Path) -> io::Result<Self::Entry>;

    /// Opens the file at `path`. `path` must be absolute.
    ///
    /// # Errors
    ///
    /// In addition to the error conditions for `open()`, this method returns an
    /// error kind of `Other` if the entry at `path` is not a regular file.
    fn open_file(&mut self, path: &Path) -> io::Result<Self::File> {
        self.open(path)?
            .into_file()
            .ok_or(io::Error::new(io::ErrorKind::Other, "not a regular file"))
    }

    /// Opens the directory at `path`. `path` must be absolute.
    ///
    /// # Errors
    ///
    /// In addition to the error conditions for `open()`, this method returns an
    /// error kind of `Other` if the entry at `path` is not a directory.
    fn open_dir(&mut self, path: &Path) -> io::Result<Self::Dir> {
        self.open(path)?
            .into_dir()
            .ok_or(io::Error::new(io::ErrorKind::Other, "not a directory"))
    }

    fn new_file(&mut self, name: String) -> io::Result<Self::File>;

    fn new_dir(&mut self, name: String) -> io::Result<Self::Dir>;
}
