use crate::stream::ancillary_buffer::AncillaryBuffer;
use crate::stream::fd_buffer::FdBuffer;
use crate::stream::read_buffer::ReadBuffer;
use rustix::io::{Errno, retry_on_intr};
use rustix::net::{RecvFlags, SendFlags, recvmsg, send, sendmsg};
use std::io::{IoSlice, IoSliceMut};
use std::os::fd::{AsFd, BorrowedFd};
use std::os::unix::net::UnixStream;

// TODO: bikeshed
pub struct RawStream {
    inner: UnixStream,
}

impl RawStream {
    // TODO: protect against invalid FDs
    #[inline]
    pub fn wrap(stream: UnixStream) -> RawStream {
        RawStream { inner: stream }
    }

    pub fn connect_to_display_server() -> std::io::Result<RawStream> {
        // TODO: other things to set up on this socket maybe?
        let stream = UnixStream::connect("/run/user/1000/wayland-0")?;

        Ok(Self::wrap(stream))
    }

    pub fn read(
        &self,
        read_buffer: &mut ReadBuffer,
        fd_buffer: &mut FdBuffer,
        ancillary_buffer: &mut AncillaryBuffer,
    ) -> Result<(), RawStreamReadResult> {
        let flags =
            // Do not block the thread if the read operation would block for any reason.
            RecvFlags::DONTWAIT |
            // Atomically sets the close-on-exec flag on all received File Descriptors.
            // By default, all file descriptors are kept and passed down to child processes when fork/exec is called.
            // When the close-on-exec flag is set on a File Descriptor, the child's file descriptor is automatically closed when fork/exec is called,
            // effectively rendering that File Descriptor private to this process only.
            RecvFlags::CMSG_CLOEXEC;

        while read_buffer.has_space_remaining() {
            let iov = &mut [IoSliceMut::new(read_buffer.as_slice_for_writing())];

            let mut control = ancillary_buffer.as_rcv_buffer();

            // TODO: better handling of INTR?
            let msg = retry_on_intr(|| recvmsg(&self.inner, iov, &mut control, flags))?;
            // TODO: check msg flags
            fd_buffer.drain_from(control);

            if msg.bytes == 0 {
                return Err(RawStreamReadResult::Eof);
            }

            read_buffer.forward_write_head_by(msg.bytes);
        }

        Ok(())
    }

    pub fn send(
        &self,
        mut data: &[u8],
        fds: &mut &[BorrowedFd<'_>],
        ancillary_buffer: &mut AncillaryBuffer,
    ) -> (usize, Result<(), RawStreamWriteResult>) {
        let flags =
            // Do not block the thread if the read operation would block for any reason.
            SendFlags::DONTWAIT |
            // No not trigger a SIGPIPE if the other end of the socket has been closed.
            SendFlags::NOSIGNAL;

        let mut total_sent_bytes = 0;

        while !data.is_empty() {
            let sent_bytes = if fds.is_empty() {
                retry_on_intr(|| send(&self.inner, data, flags))
            } else {
                let iov = [IoSlice::new(data)];
                let mut control = ancillary_buffer.fill_to_send_and_shrink_remaining(fds);

                retry_on_intr(|| sendmsg(&self.inner, &iov, &mut control, flags))
            };

            dbg!(sent_bytes);

            let sent_bytes = match sent_bytes {
                Ok(bytes) => bytes,
                Err(e) => return (total_sent_bytes, Err(e.into())),
            };

            // Shrink the slice by however many bytes were actually sent over the socket.
            // If for some reason sent > data.len() (??), then assume everything was sent.
            data = data.get(sent_bytes..).unwrap_or(&[]);
            total_sent_bytes += sent_bytes;
        }

        (total_sent_bytes, Ok(()))
    }
}

impl AsFd for RawStream {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.inner.as_fd()
    }
}

pub enum RawStreamReadResult {
    WouldBlock,
    Eof,
}

impl From<Errno> for RawStreamReadResult {
    fn from(value: Errno) -> Self {
        if value == Errno::AGAIN || value == Errno::WOULDBLOCK {
            Self::WouldBlock
        } else {
            invalid_errno(value, "recvmsg");
        }
    }
}

#[cold]
fn unreachable(msg: &'static str) -> ! {
    unreachable!("{msg}")
}

#[cold]
fn invalid_errno(errno: Errno, call: &'static str) -> ! {
    unreachable!("Got unsupported Errno for '{call}': {errno:?}")
}

pub enum RawStreamWriteResult {
    WouldBlock,
    Disconnected,
}

impl From<Errno> for RawStreamWriteResult {
    fn from(value: Errno) -> Self {
        if value == Errno::AGAIN || value == Errno::WOULDBLOCK {
            Self::WouldBlock
        } else {
            invalid_errno(value, "sendmsg");
        }
    }
}
