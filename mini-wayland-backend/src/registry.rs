use crate::registry::object_type::ObjectType;
use crate::registry::reusable_slot_map::ReusableSlotMap;
use std::cell::RefCell;
use std::num::NonZeroU32;

pub mod object_type;
mod reusable_slot_map;

pub type ObjectId = NonZeroU32;

pub struct Registry {
    object_types: RefCell<ReusableSlotMap<ObjectType>>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            object_types: RefCell::new(ReusableSlotMap::with_capacity(16)),
        }
    }

    const OFFSET: NonZeroU32 = NonZeroU32::new(2).unwrap();

    pub fn register_new(&self, object_type: ObjectType) -> ObjectId {
        let mut object_types = self.object_types.borrow_mut();
        let idx = object_types.insert(object_type);

        Self::OFFSET.checked_add(idx).unwrap() // TODO: better panic
    }

    pub fn unregister(&self, object_id: ObjectId) -> Option<ObjectType> {
        if object_id.get() == 1 {
            return Some(ObjectType::WlDisplay);
        }

        let mut object_types = self.object_types.borrow_mut();
        object_types.remove(object_id.get() - 2) // TODO: check underflow
    }

    pub fn get_object_type(&self, object_id: ObjectId) -> Option<ObjectType> {
        if object_id.get() == 1 {
            return Some(ObjectType::WlDisplay);
        }

        let object_types = self.object_types.borrow();
        object_types.get(object_id.get() - 2).copied()
    }
}
