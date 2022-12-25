use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;

use core::cmp::min;


use filesystem::fs2::File2;

use shim::{io, ioerr};
use shim::io::{Seek, SeekFrom};

use crate::multiprocessing::mutex::Mutex;

pub(crate) struct Pipe(Vec<u8>);

pub(crate) enum PipeResource {
    Writer(Arc<Mutex<Pipe>>),
    Reader(Arc<Mutex<Pipe>>),
}

impl PipeResource {
    pub(crate) fn new_pair() -> (Self, Self) {
        let pipe = Arc::new(Mutex::new(Pipe(Vec::new())));
        let writer = PipeResource::Writer(pipe.clone());
        let reader = PipeResource::Reader(pipe);
        (writer, reader)
    }
}

impl Drop for PipeResource {
    fn drop(&mut self) {}
}

impl File2 for PipeResource {
    fn duplicate(&mut self) -> io::Result<Box<dyn File2>> {
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
            PipeResource::Reader(pipe_arc) => {
                let mut pipe = pipe_arc.lock();
                let amount = min(pipe.0.len(), buf.len());
                buf[..amount].copy_from_slice(&pipe.0.as_slice()[..amount]);
                pipe.0.drain(0..amount);
                Ok(amount)
            }
        }
    }
}

impl io::Write for PipeResource {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            PipeResource::Writer(pipe_arc) => {
                let mut pipe = pipe_arc.lock();
                pipe.0.extend_from_slice(buf);
                Ok(buf.len())
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