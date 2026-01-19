use std::cell::RefCell;
use std::rc::Rc;

mod ancillary_buffer;
mod fd_buffer;
mod raw_stream;
mod read_buffer;

use crate::executor::WaylandExecutor;
use crate::stream::ancillary_buffer::AncillaryBuffer;
use crate::stream::fd_buffer::FdBuffer;
use crate::stream::read_buffer::ReadBuffer;
pub use raw_stream::RawStream;

pub struct WaylandStream {
    inner: RawStream,

    read_bufs: RefCell<(ReadBuffer, FdBuffer)>,
    ancillary_buffer: RefCell<AncillaryBuffer>,
}

impl WaylandStream {
    pub fn wrap(inner: RawStream) -> WaylandStream {
        Self {
            inner,
            read_bufs: RefCell::new((ReadBuffer::new(4096), FdBuffer::new())),
            ancillary_buffer: RefCell::new(AncillaryBuffer::new()),
        }
    }
    // TODO: errors
    async fn send_message(&self) {
        todo!()
    }

    pub fn stream(&self) -> &RawStream {
        &self.inner
    }

    fn read(&self) {
        let mut read_bufs = self.read_bufs.borrow_mut();
        let (read_buf, fd_buf) = &mut *read_bufs;
        let mut ancillary_buffer = self.ancillary_buffer.borrow_mut();

        // TODO: errors?
        let _ = self.inner.read(read_buf, fd_buf, &mut ancillary_buffer);
    }

    pub fn read_and_dispatch<H: MessageHandler, SH>(
        &self,
        handler: &Rc<H>,
        executor: &WaylandExecutor<H, SH>,
    ) {
        self.read();

        while let Some(msg) = self.next_message() {
            executor.spawn(Rc::clone(handler).handle_message(msg));
        }
    }

    fn next_message(&self) -> Option<Box<[u8]>> {
        let mut read_bufs = self.read_bufs.borrow_mut();
        let (read_buf, fd_buf) = &mut *read_bufs;
        let slice = read_buf.unread();
        let header_chunk = slice.as_chunks::<8>().0.first()?;

        // TODO: handle FDs
        let header = RawMessageHeader::parse(header_chunk);

        let msg_data = slice.get(0..header.msg_len as usize)?;

        let msg = msg_data.to_vec().into_boxed_slice();

        read_buf.forward_read_head_by(msg.len());

        Some(msg)
    }
}

pub trait MessageHandler {
    fn handle_message(self: Rc<Self>, buf: Box<[u8]>) -> impl Future<Output = ()> + 'static;
    fn closed(self: Rc<Self>) -> impl Future<Output = ()> + 'static;
}

/// A Fatal error has occured somewhere in the wire protocol.
/// All further communications on this socket are compromised, and the socket will now be closed.
pub struct FatalStreamError;

pub struct RawMessageHeader {
    pub object_id: u32,
    pub opcode: u16,
    pub msg_len: u16,
}

impl RawMessageHeader {
    pub fn parse(data: &[u8; 8]) -> Self {
        let object_id = u32::from_ne_bytes([data[0], data[1], data[2], data[3]]);
        let p2 = u32::from_ne_bytes([data[4], data[5], data[6], data[7]]);
        let opcode = (p2 & 0xffff) as u16;
        let msg_len = (p2 >> 16) as u16;

        Self {
            object_id,
            opcode,
            msg_len,
        }
    }
}
