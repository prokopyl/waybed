use crate::executor::WaylandExecutor;
use std::io;
use std::os::unix::net::UnixListener;

pub struct RawServer {
    //inner: UnixListener,
}

impl RawServer {
    pub fn bind() -> io::Result<Self> {
        // let listener = UnixListener::bind("/run/user/1000/proko-waybed-proxy")?;

        Ok(Self {})
    }
}

pub struct WaylandServer<SH> {
    handler: SH,
}

impl<SH> WaylandServer<SH> {
    pub fn wrap<H>(executor: &WaylandExecutor<H, SH>, raw: RawServer, handler: SH) -> Self {
        Self { handler }
    }
}

pub trait ServerHandler {}
