use crate::stream::raw_stream::RawStream;
use rustix::net::{RecvAncillaryBuffer, SendAncillaryBuffer};
use std::mem::MaybeUninit;
use std::os::fd::BorrowedFd;

mod ancillary_buffer;
mod fd_buffer;
mod raw_stream;
mod read_buffer;

pub struct WaylandStream {
    inner: RawStream,
}

impl WaylandStream {}
