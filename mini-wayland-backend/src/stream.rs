use crate::executor::WaylandExecutor;

mod ancillary_buffer;
mod fd_buffer;
mod raw_stream;
mod read_buffer;

pub use raw_stream::RawStream;

pub struct WaylandStream {
    inner: RawStream,
}

impl WaylandStream {
    // TODO: errors
    async fn send_message(&self) {
        todo!()
    }
}

pub trait MessageHandler {
    async fn handle_message(&self) -> Result<(), FatalStreamError>;
    fn closed(self) -> impl Future<Output = ()> + 'static;
}

/// A Fatal error has occured somewhere in the wire protocol.
/// All further communications on this socket are compromised, and the socket will now be closed.
pub struct FatalStreamError;
