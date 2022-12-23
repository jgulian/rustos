use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::borrow::Borrow;
use core::cmp::min;
use core::ops::DerefMut;

use filesystem::fs2::File2;
use kernel_api::{OsError, OsResult};
use shim::{io, ioerr};
use shim::io::{Seek, SeekFrom};

use crate::multiprocessing::mutex::Mutex;

pub(crate) type ResourceId = usize;

pub(crate) enum Resource {
    File(Box<dyn File2>),
}

pub(crate) struct ResourceEntry {
    pub(crate) id: ResourceId,
    pub(crate) resource: Resource,
}

pub(crate) struct ResourceList {
    list: Vec<ResourceEntry>,
}

impl ResourceList {
    pub(crate) fn new() -> Self {
        ResourceList {
            list: Vec::new(),
        }
    }

    pub(crate) fn insert(&mut self, resource: Resource) -> ResourceId {
        let index = self.list.iter().enumerate()
            .filter_map(|(i, resource)| {
                if i != resource.id {
                    Some(i)
                } else {
                    None
                }
            }).next().unwrap_or(0);
        self.list.insert(index, ResourceEntry {
            id: index,
            resource,
        });

        index
    }

    pub(crate) fn insert_with_id(&mut self, id: ResourceId, resource: Resource) -> OsResult<()> {
        let index = self.list.iter().enumerate()
            .filter_map(|(i, resource)| {
                if i >= resource.id {
                    Some(i)
                } else {
                    None
                }
            }).next();

        match index {
            None => {
                self.list.push(ResourceEntry {
                    id,
                    resource,
                });

                Ok(())
            }
            Some(idx) => {
                if self.list[idx].id == id {
                    return Err(OsError::UnknownResourceId);
                }

                self.list.insert(idx, ResourceEntry {
                    id,
                    resource,
                });

                Ok(())
            }
        }
    }

    pub(crate) fn remove(&mut self, id: ResourceId) -> OsResult<()> {
        let index = self.list.iter().enumerate().filter_map(|(i, res)|
            { if res.id == id { Some(i) } else { None } }).next();
        match index {
            None => {
                Err(OsError::UnknownResourceId)
            }
            Some(idx) => {
                self.list.remove(idx);
                Ok(())
            }
        }
    }

    pub(crate) fn get(&mut self, id: ResourceId) -> OsResult<&mut Resource> {
        self.list.iter_mut().filter_map(|resource| {
            if resource.id == id {
                None
            } else {
                Some(&mut resource.resource)
            }
        }).next().ok_or(OsError::UnknownResourceId)
    }
}
