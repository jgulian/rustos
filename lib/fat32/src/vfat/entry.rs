use alloc::string::String;
use filesystem;
use crate::vfat::{Dir, File, Metadata, VFatHandle};

// You can change this definition if you want
#[derive(Debug)]
pub enum Entry<HANDLE: VFatHandle> {
    File(File<HANDLE>),
    Dir(Dir<HANDLE>),
}

// TODO: Implement any useful helper methods on `Entry`.

impl<HANDLE: VFatHandle> filesystem::Entry for Entry<HANDLE> {
    type File = File<HANDLE>;
    type Dir = Dir<HANDLE>;
    type Metadata = Metadata;

    fn name(&self) -> &str {
        match self {
            Entry::File(file) => { file.name.as_str() }
            Entry::Dir(dir) => { dir.name.as_str() }
        }
    }

    fn metadata(&self) -> &Self::Metadata {
        match self {
            Entry::File(file) => { &file.metadata }
            Entry::Dir(dir) => { &dir.metadata }
        }
    }

    fn as_file(&self) -> Option<&File<HANDLE>> {
        match self {
            Entry::File(file) => { Some(&file) }
            Entry::Dir(_) => { None }
        }
    }

    fn as_dir(&self) -> Option<&Dir<HANDLE>> {
        match self {
            Entry::File(_) => { None }
            Entry::Dir(dir) => { Some(&dir) }
        }
    }

    fn into_file(self) -> Option<File<HANDLE>> {
        match self {
            Entry::File(file) => { Some(file) }
            Entry::Dir(_) => { None }
        }
    }

    fn into_dir(self) -> Option<Dir<HANDLE>> {
        match self {
            Entry::File(_) => { None }
            Entry::Dir(dir) => { Some(dir) }
        }
    }
}

impl<HANDLE: VFatHandle> Entry<HANDLE> {
    pub fn root(vfat: HANDLE) -> Entry<HANDLE> {
        Entry::<HANDLE>::Dir(Dir::<HANDLE> {
            vfat: vfat.clone(),
            first_cluster: vfat.lock(|file_system| file_system.root_cluster()),
            name: String::from("/"),
            metadata: Default::default()
        })
    }
}
