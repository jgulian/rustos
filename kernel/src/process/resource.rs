use alloc::string::String;

// TODO: resources should have the concept of being "open"
#[derive(Debug, Clone)]
pub(crate) struct Resource {
    pub(crate) descriptor: u64,
    pub(crate) path: String,
    pub(crate) seek: usize,
}