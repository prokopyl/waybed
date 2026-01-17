use crate::executor::TaskWaker;
use crate::executor::task_manager::TaskManager;
use crate::executor::task_store::TaskId;
use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};

pub struct TaskHandle {
    inner: Pin<Rc<RefCell<dyn Task>>>,
    waker: Waker,
}

impl TaskHandle {
    // TODO: future errors
    #[inline]
    pub fn new(
        future: impl Future<Output = ()> + 'static,
        task_id: TaskId,
        task_manager: Rc<TaskManager>,
    ) -> Self {
        let inner = Rc::pin(RefCell::new(future));

        let waker = TaskWaker::new(task_id, task_manager).into_waker();

        // We must not use this waker across threads
        // TODO: actually guarantee this somehow?
        let waker = unsafe { Waker::from_raw(waker) };

        Self { inner, waker }
    }

    #[inline]
    pub fn poll(&self) -> Poll<()> {
        let inner = Pin::as_ref(&self.inner);
        let mut inner = inner.borrow_mut();
        // Safety: we never move the future out, and it was pin as it comes from a Pin::Rc
        let pinned = unsafe { Pin::new_unchecked(&mut *inner) };

        let mut ctx = Context::from_waker(&self.waker);

        pinned.poll(&mut ctx)
    }
}

trait Task {
    fn poll(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<()>;
}

impl<F: Future<Output = ()>> Task for F {
    #[inline]
    fn poll(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<()> {
        self.poll(ctx)
    }
}
