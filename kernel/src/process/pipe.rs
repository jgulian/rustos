use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;

use core::cmp::min;
use filesystem::filesystem::File;

use shim::{io, ioerr};
use shim::io::{Seek, SeekFrom};
use sync::Mutex;

use crate::multiprocessing::spin_lock::SpinLock;

//TODO: this should probably be VecDeque
pub(crate) struct Pipe(Vec<u8>);

pub(crate) enum PipeResource {
    Writer(Arc<SpinLock<Pipe>>),
    Reader(Arc<SpinLock<Pipe>>),
}

impl PipeResource {
    pub(crate) fn new_pair() -> (Self, Self) {
        let pipe = Arc::new(SpinLock::new(Pipe(Vec::new())));
        let writer = PipeResource::Writer(pipe.clone());
        let reader = PipeResource::Reader(pipe);
        (writer, reader)
    }
}

impl File for PipeResource {
    fn duplicate(&mut self) -> io::Result<Box<dyn File>> {
        Ok(Box::new(match self {
            PipeResource::Writer(writer) =>
                PipeResource::Writer(writer.clone()),
            PipeResource::Reader(reader) =>
                PipeResource::Reader(reader.clone()),
        }))
    }
}

impl io::Read for PipeResource {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            PipeResource::Writer(_) => {
                ioerr!(Unsupported)
            }
            PipeResource::Reader(pipe) => {
                pipe.lock(|pipe| {
                    let amount = min(pipe.0.len(), buf.len());
                    buf[..amount].copy_from_slice(&pipe.0.as_slice()[..amount]);
                    pipe.0.drain(0..amount);
                    Ok(amount)
                }).unwrap()
            }
        }
    }
}

impl io::Write for PipeResource {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            PipeResource::Writer(pipe) => {
                pipe.lock(|pipe| {
                    pipe.0.extend_from_slice(buf);
                    Ok(buf.len())
                }).unwrap()
            }
            PipeResource::Reader(_pipe_arc) => {
                ioerr!(Unsupported)
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Seek for PipeResource {
    fn seek(&mut self, _: SeekFrom) -> io::Result<u64> {
        ioerr!(Unsupported)
    }
}