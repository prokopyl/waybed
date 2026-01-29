use std::mem;

enum Slot<T> {
    Occupied(T),
    Vacant { next_free_slot: u32 },
}

pub struct ReusableSlotMap<T> {
    slots: Vec<Slot<T>>,
    free_head: u32,
}

impl<T> ReusableSlotMap<T> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            slots: Vec::with_capacity(capacity),
            free_head: 1,
        }
    }

    pub fn get(&self, idx: u32) -> Option<&T> {
        match self.slots.get(idx as usize)? {
            Slot::Occupied(value) => Some(value),
            Slot::Vacant { .. } => None,
        }
    }

    pub fn insert(&mut self, value: T) -> u32 {
        if let Some(slot) = self.slots.get_mut(self.free_head as usize) {
            let idx = self.free_head;
            let Slot::Vacant { next_free_slot } = slot else {
                unreachable!() // TODO
            };

            self.free_head = *next_free_slot;
            *slot = Slot::Occupied(value);
            return idx;
        }

        if self.slots.len() >= u32::MAX as usize {
            panic!("SlotMap is full");
        }

        let idx = self.slots.len() as u32;
        self.slots.push(Slot::Occupied(value));
        self.free_head = idx + 1;

        idx
    }

    pub fn remove(&mut self, idx: u32) -> Option<T> {
        let slot = self.slots.get_mut(self.free_head as usize)?;
        if let Slot::Vacant { .. } = slot {
            return None;
        };

        let Slot::Occupied(value) = mem::replace(
            slot,
            Slot::Vacant {
                next_free_slot: self.free_head,
            },
        ) else {
            unreachable!()
        };

        self.free_head = idx;
        Some(value)
    }
}
