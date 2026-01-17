use crate::executor::task::TaskHandle;
use slotmap::{SlotMap, new_key_type};
use std::cell::RefCell;

new_key_type! {
    pub struct TaskId;
}

pub struct TaskStore {
    tasks: RefCell<SlotMap<TaskId, TaskHandle>>,
}

impl TaskStore {
    pub fn new() -> TaskStore {
        Self {
            tasks: RefCell::new(SlotMap::with_key()),
        }
    }

    pub fn spawn(&self, future: impl Future<Output = ()> + 'static) -> TaskId {
        todo!()
    }
}
