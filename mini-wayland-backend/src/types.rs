use crate::stream::{RawMessageHeader, Serializable, Serializer};

pub struct MessageWithHeader<M> {
    pub target_id: u32,
    pub event: M,
}

impl<M: Serializable> MessageWithHeader<M> {
    pub fn serialize(&self, serializer: &mut Serializer) {
        let header = RawMessageHeader {
            object_id: self.target_id,
            msg_len: self.event.data_len() + 8, // TODO: overflow
            opcode: M::OP_CODE,
        };

        serializer.write_raw(&header.to_bytes());
        self.event.serialize(serializer);
    }

    pub(crate) fn full_len(&self) -> u16 {
        self.event.data_len() + 8 // TODO: overflow
    }
}
