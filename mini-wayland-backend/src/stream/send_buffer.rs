use crate::MessageWithHeader;
use oneringbuf::iterators::{ConsIter, ProdIter};
use oneringbuf::{LocalHeapRB, LocalVmemRB, ORBIterator};
use std::marker::PhantomData;

pub struct SendBuffer {
    prod: ProdIter<LocalVmemRB<u8>>,
    cons: ConsIter<LocalVmemRB<u8>>,
    _no_send: PhantomData<*const ()>,
}

impl SendBuffer {
    pub fn new(capacity: usize) -> Self {
        let buf = LocalVmemRB::default(capacity);
        let (prod, cons) = buf.split();

        Self {
            prod,
            cons,
            _no_send: PhantomData,
        }
    }

    #[inline]
    pub fn forward_read_head_by(&mut self, bytes: usize) {
        let bytes = bytes.min(self.cons.available());
        // SAFETY: we just restricted bytes above to be <= avail
        unsafe { self.cons.advance(bytes) }
    }

    pub fn len_to_send(&mut self) -> usize {
        self.cons.available()
    }

    pub fn to_send(&mut self) -> &[u8] {
        self.cons.get_mut_slice_avail().unwrap_or(&mut [])
    }

    pub fn write(
        &mut self,
        serializable: &MessageWithHeader<impl Serializable>,
        offset: usize,
    ) -> usize {
        let mut serialize = Serializer::new(&mut self.prod, offset);
        serializable.serialize(&mut serialize);
        serialize.total_serialized
    }
}

pub trait Serializable {
    const OP_CODE: u16;

    fn serialize(&self, serializer: &mut Serializer<'_>);
    fn data_len(&self) -> u16;
}

pub struct Serializer<'buf> {
    buf: &'buf mut ProdIter<LocalVmemRB<u8>>,
    remaining_offset: usize,
    total_serialized: usize,
}

impl<'buf> Serializer<'buf> {
    pub fn new(buf: &'buf mut ProdIter<LocalVmemRB<u8>>, offset: usize) -> Self {
        Self {
            buf,
            remaining_offset: offset,
            total_serialized: 0,
        }
    }

    pub fn write_raw(&mut self, bytes: &[u8]) {
        if self.remaining_offset >= bytes.len() {
            self.remaining_offset -= bytes.len();
            return;
        }

        let Some(remaining) = bytes.get(self.remaining_offset..) else {
            self.remaining_offset -= bytes.len();
            return;
        };

        let size_to_send = remaining.len().min(self.buf.available());
        self.remaining_offset = self.remaining_offset.saturating_sub(size_to_send);
        self.total_serialized += size_to_send;

        self.buf.push_slice(&remaining[..size_to_send]);
    }

    pub fn write_u32(&mut self, value: u32) {
        let bytes = value.to_ne_bytes();
        self.write_raw(&bytes);
    }
}
