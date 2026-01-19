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

        executor.wrap_stream(wayland_conn, MyHandler::MainConnection {});

        let _server_socket = WaylandServer::wrap(&executor, server, MyServerHandler {});

        executor.run_until(|| !closed.get()).unwrap();

        println!("Thread done");
    });
}

enum MyHandler {
    Master { closed: Rc<Cell<bool>> },
    MainConnection {},
}

impl MessageHandler for MyHandler {
    async fn handle_message(self: Rc<Self>, buf: Box<[u8]>) {
        println!("New message! {:?}", buf);
    }

    async fn closed(self: Rc<Self>) {
        match &*self {
            MyHandler::Master { closed } => closed.set(true),
            _ => {}
        };
    }
}

struct MyServerHandler {}

impl ServerHandler for MyServerHandler {}
