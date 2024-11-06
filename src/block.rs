use crate::MAX_INDEX_LEVEL;
use core::ptr;
pub(crate) const BLOCK_VEC_SIZE: usize = size_of::<BlockVec>();

#[derive(Clone, Copy)]
pub(crate) struct Block {
    pub next: *mut Block,
}

#[derive(Clone, Copy)]
pub(crate) struct BlockVec {
    index: [usize; MAX_INDEX_LEVEL],
    pub(crate) levels: usize,
}

impl BlockVec {
    pub unsafe fn new(addr: usize, levels: usize) -> *mut Self {
        let vec = addr as *mut usize as *mut BlockVec;
        (*vec).index = [0; MAX_INDEX_LEVEL];
        (*vec).levels = levels;
        vec
    }

    pub unsafe fn from_addr(addr: usize) -> *mut Self {
        addr as *mut usize as *mut BlockVec
    }

    pub unsafe fn get(&self, index: usize) -> *mut Block {
        let ptr = self.index.as_ptr() as usize;
        if index < self.levels {
            let offset = index * size_of::<Block>();
            (ptr + offset) as *mut usize as *mut Block
        } else {
            ptr::null_mut()
        }
    }
}
