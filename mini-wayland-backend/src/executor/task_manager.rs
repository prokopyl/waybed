use crate::executor::task_store::TaskId;
use slotmap::{SecondaryMap, new_key_type};
use std::cell::RefCell;

pub struct TaskManager {
    tasks_to_wake: RefCell<SecondaryMap<TaskId, ()>>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            tasks_to_wake: RefCell::new(SecondaryMap::with_capacity(16)),
        }
    }

    pub fn wake(&self, task_id: TaskId) {
        let Ok(mut tasks) = self.tasks_to_wake.try_borrow_mut() else {
            // TODO?
            todo!();
            return;
        };

        tasks.insert(task_id, ());
    }

    pub fn drain(&self, mut handler: impl FnMut(TaskId)) {
        while let Some(task_id) = self.pop_task() {
            handler(task_id)
        }
    }

    fn pop_task(&self) -> Option<TaskId> {
        let mut tasks = self.tasks_to_wake.borrow_mut();
        let mut drain = tasks.drain();
        let task_id = drain.next();
        core::mem::forget(drain);

        task_id.map(|task_id| task_id.0)
    }
}
