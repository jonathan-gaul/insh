use std::ptr;

use super::bytecode_chunk::ByteCodeChunk;

pub(crate) struct ByteCodeChunkReader {
    first: *const u8,
    last: *const u8,
    ptr: *const u8,
}

impl ByteCodeChunkReader {
    pub fn new(chunk: &ByteCodeChunk) -> Self {
        let ptr = chunk.content.as_ptr();
        let last = ptr.wrapping_add(chunk.content.len());
        ByteCodeChunkReader {
            first: ptr,
            ptr,
            last,
        }
    }

    #[inline(always)]
    pub fn next<T>(&mut self) -> Option<T> {
        unsafe {
            if self.ptr < self.last {
                let value = ptr::read_unaligned(self.ptr as *const T);
                self.ptr = self.ptr.add(size_of::<T>());
                Some(value)
            } else {
                None
            }
        }
    }

    pub fn get_offset(&self) -> usize {
        self.ptr.wrapping_sub(self.first as usize) as usize
    }
}
