use crate::executor::TaskWaker;
use crate::executor::task::TaskHandle;
use crate::executor::task_manager::TaskManager;
use slotmap::{SlotMap, new_key_type};
use std::cell::RefCell;
use std::rc::Rc;
use std::task::Poll;

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

    fn new_task(
        &self,
        future: impl Future<Output = ()> + 'static,
        manager: Rc<TaskManager>,
    ) -> TaskId {
        self.tasks
            .borrow_mut()
            .insert_with_key(|id| TaskHandle::new(future, id, manager))
    }

    pub fn spawn(
        &self,
        future: impl Future<Output = ()> + 'static,
        manager: Rc<TaskManager>,
    ) -> TaskId {
        let task_id = self.new_task(future, manager);

        if let Poll::Ready(()) = self.poll(task_id) {
            self.delete_task(task_id);
        }

        task_id
    }

    pub fn poll(&self, task_id: TaskId) -> Poll<()> {
        match self.get_task(task_id) {
            Some(task) => task.poll(),
            None => Poll::Ready(()),
        }
    }

    fn get_task(&self, id: TaskId) -> Option<TaskHandle> {
        let tasks = self.tasks.borrow();
        tasks.get(id).map(TaskHandle::clone)
    }

    pub fn delete_task(&self, id: TaskId) {
        let mut tasks = self.tasks.borrow_mut();
        tasks.remove(id);
    }
}
