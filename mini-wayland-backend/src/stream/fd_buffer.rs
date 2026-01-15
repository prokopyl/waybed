use rustix::fd::OwnedFd;
use rustix::net::{RecvAncillaryBuffer, RecvAncillaryMessage};
use std::collections::VecDeque;

pub struct FdBuffer {
    inner: VecDeque<OwnedFd>,
}

impl FdBuffer {
    pub fn new() -> FdBuffer {
        FdBuffer {
            inner: VecDeque::with_capacity(32),
        }
    }

    pub fn push(&mut self, fd: OwnedFd) {
        self.inner.push_back(fd);
    }

    pub fn drain_from(&mut self, mut control: RecvAncillaryBuffer) {
        let received_fds = control
            .drain()
            .filter_map(|msg| match msg {
                RecvAncillaryMessage::ScmRights(fd) => Some(fd),
                _ => None,
            })
            .flatten();

        self.inner.extend(received_fds)
    }

    pub fn extend(&mut self, fds: impl IntoIterator<Item = OwnedFd>) {
        self.inner.extend(fds);
    }
}
