use crate::executor::task_manager::TaskManager;
use crate::executor::task_store::TaskId;
use crate::executor::waker::TaskWaker;
use slotmap::SlotMap;
use std::cell::RefCell;
use std::rc::Rc;

pub struct WakerStore {
    wakers: RefCell<SlotMap<TaskId, Option<Rc<TaskWaker>>>>,
}

impl WakerStore {
    pub fn new() -> Self {
        Self {
            wakers: RefCell::new(SlotMap::with_key()),
        }
    }

    pub fn lend_new_waker(&self, task_manager: &Rc<TaskManager>) -> Rc<TaskWaker> {
        let mut wakers = self.wakers.borrow_mut();

        for waker_slot in wakers.values_mut() {
            if let Some(waker) = waker_slot.take() {
                return waker;
            }
        }

        let new_slot_id = wakers.insert(None);
        TaskWaker::new(new_slot_id, Rc::clone(task_manager))
    }

    pub fn return_waker(&self, waker: Rc<TaskWaker>) {
        let mut wakers = self.wakers.borrow_mut();

        if let Some(slot) = wakers.get_mut(waker.token) {
            *slot = Some(waker);
        }
    }
}
