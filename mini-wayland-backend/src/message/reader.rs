use crate::registry::ObjectId;
use std::ffi::CStr;

pub struct Reader<'a> {
    buf: &'a [u8],
    position: usize,
}

impl<'a> Reader<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf, position: 0 }
    }

    // TODO: handle errors
    pub fn read_int(&mut self) -> i32 {
        let data = self.buf.get(self.position..self.position + 4).unwrap();
        self.position += 4;
        i32::from_ne_bytes(*data.as_array().unwrap())
    }

    pub fn read_uint(&mut self) -> u32 {
        let data = self.buf.get(self.position..self.position + 4).unwrap();
        self.position += 4;
        u32::from_ne_bytes(*data.as_array().unwrap())
    }

    pub fn read_object_id(&mut self) -> Option<ObjectId> {
        let raw_id = self.read_uint();
        ObjectId::new(raw_id)
    }

    fn align_to_next_u32(&mut self) {
        let modulo = self.position % 4;
        if modulo != 0 {
            self.position += 4 - modulo;
        }
    }

    pub fn read_str(&mut self) -> Option<String> {
        let len = self.read_uint() as usize; // TODO: do not update position if read fails?

        if len == 0 {
            return None;
        }

        let str_buf = &self.buf[self.position..self.position + len];

        self.position += len;
        self.align_to_next_u32();

        let null_terminated_str = CStr::from_bytes_with_nul(str_buf).unwrap();

        Some(null_terminated_str.to_str().unwrap().to_string())
    }
}
