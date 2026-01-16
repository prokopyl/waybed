use crate::proxy_thread::start_proxy_thread;
use mini_wayland_backend::executor::RawWaylandExecutor;
use mini_wayland_backend::server::RawServer;
use mini_wayland_backend::stream::RawStream;
use std::os::unix::net::{UnixListener, UnixStream};

mod proxy_thread;

pub struct WaybedProxy {
    pub wayland_socket: UnixStream,
    // TODO: expose path for clients to connect to.
}

impl WaybedProxy {
    pub fn create() -> Self {
        let executor = RawWaylandExecutor::new().unwrap();
        // TODO: unwrap
        let (a, b) = UnixStream::pair().unwrap();
        // TODO: unwrap
        let connect = RawStream::connect_to_display_server().unwrap();

        let server = RawServer::bind().unwrap();

        // We have set up everything now, everything that could fail has either gone through or failed.
        // We can set up the proxy thread now.

        start_proxy_thread(connect, RawStream::wrap(b), server, executor);

        WaybedProxy { wayland_socket: a }
    }
}
