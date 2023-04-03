use alloc::boxed::Box;
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::fmt;
use core::fmt::Formatter;

use filesystem::filesystem::File;
use kernel_api::{OsError, OsResult};

#[derive(Clone, Copy, PartialOrd, PartialEq, Debug)]
pub struct ResourceId(u64);

impl From<u64> for ResourceId {
    fn from(value: u64) -> Self {
        ResourceId(value)
    }
}

impl From<ResourceId> for u64 {
    fn from(val: ResourceId) -> Self {
        val.0
    }
}

impl PartialEq<u64> for ResourceId {
    fn eq(&self, other: &u64) -> bool {
        self.0.eq(other)
    }
}

impl PartialOrd<u64> for ResourceId {
    fn partial_cmp(&self, other: &u64) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

pub enum Resource {
    File(Box<dyn File>),
}

pub struct ResourceEntry {
    pub(crate) id: ResourceId,
    pub(crate) resource: Resource,
}

impl fmt::Debug for ResourceEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResourceEntry")
            .field("id", &self.id)
            .finish()
    }
}

#[derive(Debug)]
pub struct ResourceList {
    list: Vec<ResourceEntry>,
}

impl ResourceList {
    pub(crate) fn new() -> Self {
        ResourceList { list: Vec::new() }
    }

    pub(crate) fn insert(&mut self, resource: Resource) -> ResourceId {
        let index = self
            .list
            .iter()
            .enumerate()
            .filter_map(|(i, resource)| {
                if resource.id != i as u64 {
                    Some(i)
                } else {
                    None
                }
            })
            .next()
            .unwrap_or(0);
        let id = ResourceId::from(index as u64);
        self.list.insert(index, ResourceEntry { id, resource });
        id
    }

    pub(crate) fn insert_with_id(&mut self, id: ResourceId, resource: Resource) -> OsResult<()> {
        let index = self
            .list
            .iter()
            .enumerate()
            .filter_map(|(i, resource)| {
                if resource.id <= i as u64 {
                    Some(i)
                } else {
                    None
                }
            })
            .next();

        match index {
            None => {
                self.list.push(ResourceEntry { id, resource });

                Ok(())
            }
            Some(idx) => {
                if self.list[idx].id == id {
                    return Err(OsError::UnknownResourceId);
                }

                self.list.insert(idx, ResourceEntry { id, resource });

                Ok(())
            }
        }
    }

    pub(crate) fn remove(&mut self, id: ResourceId) -> OsResult<()> {
        let index = self
            .list
            .iter()
            .enumerate()
            .filter_map(|(i, res)| if res.id == id { Some(i) } else { None })
            .next();
        match index {
            None => Err(OsError::UnknownResourceId),
            Some(idx) => {
                self.list.remove(idx);
                Ok(())
            }
        }
    }

    pub(crate) fn get(&mut self, id: ResourceId) -> OsResult<&mut Resource> {
        self.list
            .iter_mut()
            .filter_map(|resource| {
                if resource.id == id {
                    Some(&mut resource.resource)
                } else {
                    None
                }
            })
            .next()
            .ok_or(OsError::UnknownResourceId)
    }

    pub(crate) fn duplicate(&mut self) -> OsResult<ResourceList> {
        let list: Vec<ResourceEntry> = self
            .list
            .iter_mut()
            .map_while(|entry| {
                let resource = match &mut entry.resource {
                    Resource::File(ref mut file) => Resource::File(file.duplicate().ok()?),
                };

                Some(ResourceEntry {
                    id: entry.id,
                    resource,
                })
            })
            .collect();

        Ok(ResourceList { list })
    }
}
