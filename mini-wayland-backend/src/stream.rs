use std::cell::RefCell;
use std::mem;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};

mod ancillary_buffer;
mod fd_buffer;
mod raw_stream;
mod read_buffer;
mod send_buffer;

use crate::MessageWithHeader;
use crate::executor::WaylandExecutor;
use crate::message::Message;
use crate::registry::Registry;
use crate::stream::ancillary_buffer::AncillaryBuffer;
use crate::stream::fd_buffer::FdBuffer;
use crate::stream::raw_stream::RawStreamWriteResult;
use crate::stream::read_buffer::ReadBuffer;
use crate::stream::send_buffer::SendBuffer;
pub use raw_stream::RawStream;
pub use send_buffer::{Serializable, Serializer};

pub struct WaylandStream {
    inner: RawStream,

    read_bufs: RefCell<(ReadBuffer, FdBuffer)>,
    send_bufs: RefCell<SendBuffer>,
    ancillary_buffer: RefCell<AncillaryBuffer>,

    wakers: RefCell<Vec<Waker>>,
    registry: Registry,
}

impl WaylandStream {
    pub fn wrap(inner: RawStream) -> WaylandStream {
        Self {
            inner,
            read_bufs: RefCell::new((ReadBuffer::new(4096), FdBuffer::new())),
            send_bufs: RefCell::new(SendBuffer::new(4096)),
            ancillary_buffer: RefCell::new(AncillaryBuffer::new()),
            wakers: RefCell::new(Vec::with_capacity(8)),
            registry: Registry::new(),
        }
    }

    pub fn send_message<'s, 'b, M: Serializable>(
        &'s self,
        msg: &'b MessageWithHeader<M>,
    ) -> StreamSendFuture<'s, 'b, M> {
        StreamSendFuture {
            already_sent_bytes: 0,
            msg,
            stream: self,
            total_msg_len: msg.full_len(),
        }
    }

    pub fn flush(&self) -> FlushFuture<'_> {
        FlushFuture { stream: self }
    }

    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    fn append_to_buf(&self, msg: &MessageWithHeader<impl Serializable>, offset: usize) -> usize {
        let mut send_bufs = self.send_bufs.borrow_mut();
        send_bufs.write(msg, offset)
    }

    fn flush_raw(&self) -> Result<(), RawStreamWriteResult> {
        let mut send_bufs = self.send_bufs.borrow_mut();
        if send_bufs.len_to_send() == 0 {
            return Ok(());
        }

        let data = send_bufs.to_send();
        dbg!(data);
        let mut ancillary_buffer = self.ancillary_buffer.borrow_mut();
        let (sent_bytes, res) = self.inner.send(data, &mut &[][..], &mut ancillary_buffer);

        send_bufs.forward_read_head_by(sent_bytes);

        res
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

    fn register_waker(&self, waker: &Waker) {
        let mut wakers = self.wakers.borrow_mut();
        wakers.push(waker.clone());
    }

    fn get_all_wakers(&self) -> Vec<Waker> {
        let mut wakers = self.wakers.borrow_mut();
        mem::take(&mut *wakers)
    }

    pub fn dispatch_send_wakers(&self) {
        let wakers = self.get_all_wakers();

        for waker in wakers {
            waker.wake();
        }
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

    fn next_message(&self) -> Option<Message> {
        let mut read_bufs = self.read_bufs.borrow_mut();
        // TODO: handle FDs

        let (read_buf, fd_buf) = &mut *read_bufs;

        let (header, remaining) = read_buf.unread().split_at_checked(8)?;
        let header = RawMessageHeader::parse(header.as_array()?);
        let msg_data_len = header.msg_len.checked_sub(8).unwrap().into(); // TODO: panic
        let msg_data = remaining.get(..msg_data_len)?;

        let msg = Message::parse(header, msg_data, &self.registry); // TODO: panics

        read_buf.forward_read_head_by(header.msg_len.into());

        Some(msg)
    }
}

pub trait MessageHandler {
    fn handle_message(self: Rc<Self>, msg: Message) -> impl Future<Output = ()> + 'static;
    fn closed(self: Rc<Self>) -> impl Future<Output = ()> + 'static;
}

/// A Fatal error has occured somewhere in the wire protocol.
/// All further communications on this socket are compromised, and the socket will now be closed.
pub struct FatalStreamError;

#[derive(Copy, Clone, Debug)]
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

    pub fn to_bytes(&self) -> [u8; 8] {
        let p1 = self.object_id.to_ne_bytes();
        let p2 = ((self.msg_len as u32) << 16) | self.opcode as u32;
        let p2 = p2.to_ne_bytes();

        [p1[0], p1[1], p1[2], p1[3], p2[0], p2[1], p2[2], p2[3]]
    }
}

pub struct StreamSendFuture<'s, 'm, M> {
    stream: &'s WaylandStream,
    msg: &'m MessageWithHeader<M>,
    already_sent_bytes: u16,
    total_msg_len: u16,
}

impl<M: Serializable> Future for StreamSendFuture<'_, '_, M> {
    type Output = Result<(), SendError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = Pin::into_inner(self);

        loop {
            let written = this
                .stream
                .append_to_buf(this.msg, this.already_sent_bytes as usize);
            this.already_sent_bytes += written as u16; // TODO: casts/overflow

            if this.already_sent_bytes == this.total_msg_len {
                return Poll::Ready(Ok(()));
            }

            match this.stream.flush_raw() {
                Ok(()) => {}
                Err(RawStreamWriteResult::Disconnected) => {
                    return Poll::Ready(Err(SendError::Disconnected));
                }
                Err(RawStreamWriteResult::WouldBlock) => {
                    this.stream.register_waker(cx.waker());
                    return Poll::Pending;
                }
            }
        }
    }
}

pub struct FlushFuture<'s> {
    stream: &'s WaylandStream,
}

impl Future for FlushFuture<'_> {
    type Output = Result<(), SendError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = Pin::into_inner(self);

        match this.stream.flush_raw() {
            Ok(()) => Poll::Ready(Ok(())),
            Err(RawStreamWriteResult::Disconnected) => Poll::Ready(Err(SendError::Disconnected)),
            Err(RawStreamWriteResult::WouldBlock) => {
                // TODO: register stream for interest in sending
                this.stream.register_waker(cx.waker());
                Poll::Pending
            }
        }
    }
}

#[derive(Debug)]
pub enum SendError {
    Disconnected,
}
