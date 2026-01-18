use oneringbuf::iterators::{ConsIter, WorkIter};
use oneringbuf::{LocalVmemRB, LocalVmemRBMut, ORBIterator};
use std::marker::PhantomData;

pub struct ReadBuffer {
    work: WorkIter<LocalVmemRBMut<u8>>,
    cons: ConsIter<LocalVmemRBMut<u8>>,
    _no_send: PhantomData<*mut ()>,
}

impl ReadBuffer {
    pub fn new(capacity: usize) -> Self {
        let buf = LocalVmemRBMut::default(capacity);
        let (_, work, cons) = buf.split_mut();

        Self {
            work,
            cons,
            _no_send: PhantomData,
        }
    }

    #[inline]
    pub fn forward_write_head_by(&mut self, bytes: usize) {
        let bytes = bytes.min(self.work.available());
        // SAFETY: we just restricted bytes above to be <= avail
        unsafe { self.work.advance(bytes) }
    }

    pub fn as_slice_for_writing(&mut self) -> &mut [u8] {
        self.work.get_mut_slice_avail().unwrap_or(&mut [])
    }

    pub fn as_slices(&self) -> (&[u8], &[u8]) {
        todo!()
    }
}
