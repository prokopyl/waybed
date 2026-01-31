use mini_wayland_backend::MessageWithHeader;
use mini_wayland_backend::executor::{RawWaylandExecutor, WaylandExecutor};
use mini_wayland_backend::message::Message;
use mini_wayland_backend::server::{RawServer, ServerHandler, WaylandServer};
use mini_wayland_backend::stream::{MessageHandler, RawStream, Serializable, Serializer};
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

        let wayland_conn = executor.wrap_stream(wayland_conn, MyHandler::MainConnection {});

        executor.spawn(async move {
            wayland_conn
                .send_message(&MessageWithHeader {
                    target_id: 1,
                    event: GetRegistry { new_id: 42 },
                })
                .await
                .unwrap();

            wayland_conn.flush().await.unwrap();
            eprintln!("Sent!")
        });

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
    async fn handle_message(self: Rc<Self>, msg: Message) {
        println!("New message! {:?}", msg);
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

pub struct GetRegistry {
    new_id: u32,
}

impl Serializable for GetRegistry {
    const OP_CODE: u16 = 1;

    fn serialize(&self, serializer: &mut Serializer<'_>) {
        serializer.write_u32(self.new_id);
    }

    fn data_len(&self) -> u16 {
        4
    }
}
