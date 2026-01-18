use crate::executor::task_manager::TaskManager;
use crate::executor::task_store::{TaskId, TaskStore};
use crate::stream::{MessageHandler, RawStream};
use rustix::event::epoll;
use rustix::event::epoll::{CreateFlags, Event, EventData, EventFlags};
use slotmap::{KeyData, SecondaryMap, SlotMap, new_key_type};
use std::io;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::os::fd::{AsFd, AsRawFd, OwnedFd};
use std::rc::Rc;

pub mod task;
mod task_manager;
mod task_store;
mod waker;
mod waker_store;

pub use waker::TaskWaker;

pub struct RawWaylandExecutor {
    epoll: OwnedFd,
}

const EVENT_STREAM_CLOSED: EventFlags = EventFlags::RDHUP
    .union(EventFlags::ERR)
    .union(EventFlags::HUP);

impl RawWaylandExecutor {
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            epoll: epoll::create(CreateFlags::CLOEXEC)?,
        })
    }

    pub(crate) fn register_socket(&self, socket: &RawStream, token: u64) -> rustix::io::Result<()> {
        let listen_for = EventFlags::IN | EventFlags::OUT | EVENT_STREAM_CLOSED;

        // TODO: handle errors
        epoll::add(&self.epoll, socket, EventData::new_u64(token), listen_for)
    }

    pub(crate) fn unregister_socket(&self, socket: &RawStream) -> rustix::io::Result<()> {
        epoll::delete(&self.epoll, socket)
    }

    pub(crate) fn wait<'b>(
        &self,
        events_buf: &'b mut [MaybeUninit<Event>],
    ) -> rustix::io::Result<&'b [Event]> {
        // TODO: handle EINTR
        Ok(epoll::wait(&self.epoll, events_buf, None)?.0)
    }
}

new_key_type! {
    pub struct WaylandStreamId;
}

pub struct WaylandExecutor<H, SH> {
    inner: RawWaylandExecutor,
    streams: SlotMap<WaylandStreamId, RawStream>,
    handlers: SecondaryMap<WaylandStreamId, H>,
    tasks: TaskStore,
    task_manager: Rc<TaskManager>,
    _handler: PhantomData<fn() -> (H, SH)>,
}

impl<H: MessageHandler, SH> WaylandExecutor<H, SH> {
    pub fn new(raw: RawWaylandExecutor) -> Self {
        Self {
            inner: raw,
            streams: SlotMap::with_capacity_and_key(8),
            handlers: SecondaryMap::with_capacity(8),
            tasks: TaskStore::new(),
            task_manager: Rc::new(TaskManager::new()),
            _handler: PhantomData,
        }
    }

    pub fn wrap_stream(&mut self, raw: RawStream, handler: H) {
        let key = self.streams.insert(raw);
        let raw = &self.streams[key];

        // TODO: handle error
        self.inner.register_socket(raw, key.0.as_ffi()).unwrap();
        self.handlers.insert(key, handler);
    }

    pub fn run_until(&mut self, until: impl Fn() -> bool) -> rustix::io::Result<()> {
        let mut buf = vec![MaybeUninit::zeroed(); 1024].into_boxed_slice();

        while until() {
            let events = self.inner.wait(&mut buf)?;
            for event in events {
                // Stuff to read from socket
                if !event.flags.intersection(EventFlags::IN).is_empty() {
                    println!("Read available for {}!", event.data.u64())
                }

                // Socket closed
                if !event.flags.intersection(EVENT_STREAM_CLOSED).is_empty() {
                    println!("Closed! {}", event.data.u64());
                    // TODO: read FD to end
                    let socket_key = WaylandStreamId::from(KeyData::from_ffi(event.data.u64()));

                    if let Some(socket) = self.streams.remove(socket_key) {
                        self.inner.unregister_socket(&socket)?;
                    }

                    if let Some(handler) = self.handlers.remove(socket_key) {
                        self.spawn(handler.closed());
                    }
                }
            }
        }

        Ok(())
    }

    fn spawn(&self, future: impl Future<Output = ()> + 'static) -> TaskId {
        self.tasks.spawn(future, Rc::clone(&self.task_manager))
    }
}
