use mini_wayland_backend::executor::{RawWaylandExecutor, WaylandExecutor};
use mini_wayland_backend::server::{RawServer, ServerHandler, WaylandServer};
use mini_wayland_backend::stream::{FatalStreamError, MessageHandler, RawStream, WaylandStream};
use std::cell::Cell;
use std::rc::Rc;

pub fn start_proxy_thread(
    wayland_conn: RawStream,
    master_client: RawStream,
    server: RawServer,
    raw_wayland_executor: RawWaylandExecutor,
) {
    // TODO: customize proxy thread name, etc.
    std::thread::spawn(move || {
        let mut executor = WaylandExecutor::new(raw_wayland_executor);

        let closed = Rc::new(Cell::new(false));

        executor.wrap_stream(
            master_client,
            MyHandler::Master {
                closed: closed.clone(),
            },
        );

        let _server_socket = WaylandServer::wrap(&executor, server, MyServerHandler {});

        executor.run_until(|| !closed.get()).unwrap();
    });
}

enum MyHandler {
    Master { closed: Rc<Cell<bool>> },
}

impl MessageHandler for MyHandler {
    async fn handle_message(&self) -> Result<(), FatalStreamError> {
        todo!()
    }

    async fn closed(self) {
        match self {
            MyHandler::Master { closed } => closed.set(true),
        }
    }
}

struct MyServerHandler {}

impl ServerHandler for MyServerHandler {}
