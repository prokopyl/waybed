use crate::executor::WaylandStreamId;
use crate::executor::task_manager::TaskManager;
use crate::executor::task_store::TaskId;
use std::mem::ManuallyDrop;
use std::rc::Rc;
use std::task::{RawWaker, RawWakerVTable, Waker};

pub struct TaskWaker {
    pub token: TaskId,
    task_manager: Rc<TaskManager>,
}

impl TaskWaker {
    #[inline]
    pub fn new(token: TaskId, task_manager: Rc<TaskManager>) -> Rc<Self> {
        Rc::new(Self {
            token,
            task_manager,
        })
    }

    #[inline]
    pub fn into_waker(self: Rc<Self>) -> RawWaker {
        RawWaker::new(Rc::into_raw(self) as *const (), &RAW_WAKER_VTABLE)
    }

    fn wake(&self) {
        self.task_manager.wake(self.token);
    }
}

static RAW_WAKER_VTABLE: RawWakerVTable = {
    unsafe fn clone_waker(waker: *const ()) -> RawWaker {
        unsafe { Rc::increment_strong_count(waker as *const TaskWaker) };
        RawWaker::new(waker, &RAW_WAKER_VTABLE)
    }

    unsafe fn wake_raw_by_ref(waker: *const ()) {
        let waker = unsafe { ManuallyDrop::new(Rc::from_raw(waker as *const TaskWaker)) };
        waker.wake();
    }

    unsafe fn wake_raw(waker: *const ()) {
        let waker = unsafe { Rc::from_raw(waker as *const TaskWaker) };
        waker.wake();
    }

    unsafe fn drop(waker: *const ()) {
        unsafe { Rc::decrement_strong_count(waker as *const TaskWaker) };
    }

    RawWakerVTable::new(clone_waker, wake_raw, wake_raw_by_ref, drop)
};
