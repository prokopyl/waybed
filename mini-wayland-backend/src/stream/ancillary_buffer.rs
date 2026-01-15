use rustix::net::{RecvAncillaryBuffer, SendAncillaryBuffer};
use std::mem::MaybeUninit;
use std::os::fd::BorrowedFd;

pub struct AncillaryBuffer {
    inner: Box<[MaybeUninit<u8>]>,
}

impl AncillaryBuffer {
    const MAX_FDS: usize = 1024;
    pub fn new() -> AncillaryBuffer {
        Self {
            inner: vec![MaybeUninit::zeroed(); rustix::cmsg_space!(ScmRights(Self::MAX_FDS))]
                .into_boxed_slice(),
        }
    }

    pub fn as_rcv_buffer(&mut self) -> RecvAncillaryBuffer<'_> {
        RecvAncillaryBuffer::new(&mut self.inner)
    }

    pub fn fill_to_send_and_shrink_remaining<'buf, 'slice, 'fd>(
        &'buf mut self,
        fds: &mut &'slice [BorrowedFd<'fd>],
    ) -> SendAncillaryBuffer<'buf, 'slice, 'fd> {
        todo!()
    }
}
