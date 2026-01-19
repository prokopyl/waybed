use oneringbuf::iterators::{ConsIter, ProdIter, WorkIter};
use oneringbuf::{LocalVmemRB, LocalVmemRBMut, ORBIterator};
use std::marker::PhantomData;

pub struct ReadBuffer {
    prod: ProdIter<LocalVmemRB<u8>>,
    cons: ConsIter<LocalVmemRB<u8>>,
    _no_send: PhantomData<*mut ()>,
}

impl ReadBuffer {
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
    pub fn forward_write_head_by(&mut self, bytes: usize) {
        let bytes = bytes.min(self.prod.available());
        // SAFETY: we just restricted bytes above to be <= avail
        unsafe { self.prod.advance(bytes) }
    }

    #[inline]
    pub fn forward_read_head_by(&mut self, bytes: usize) {
        let bytes = bytes.min(self.cons.available());
        // SAFETY: we just restricted bytes above to be <= avail
        unsafe { self.cons.advance(bytes) }
    }

    pub fn as_slice_for_writing(&mut self) -> &mut [u8] {
        self.prod.get_mut_slice_avail().unwrap_or(&mut [])
    }

    pub fn has_space_remaining(&mut self) -> bool {
        self.prod.available() > 0
    }

    pub fn has_unread_data(&mut self) -> bool {
        self.cons.available() > 0
    }

    pub fn unread(&mut self) -> &mut [u8] {
        self.cons.get_mut_slice_avail().unwrap_or(&mut [])
    }
}
