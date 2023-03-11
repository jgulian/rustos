use alloc::rc::Rc;
use shim::io;
use shim::io::SeekFrom;
use crate::cluster::Cluster;
use crate::vfat::VFat;

#[derive(Clone)]
pub(crate) struct Chain {
    vfat: Rc<VFat>,
    position: u64,
    first_cluster: Cluster,
    current_cluster: Cluster,
}

impl Chain {
    pub(crate) fn new(vfat: Rc<VFat>) -> io::Result<Self> {

    }
}

impl io::Read for Chain {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        todo!()
    }
}

impl io::Write for Chain {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        todo!()
    }

    fn flush(&mut self) -> io::Result<()> {
        todo!()
    }
}

impl io::Seek for Chain {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        todo!()
    }
}