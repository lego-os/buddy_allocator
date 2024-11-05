use core::ptr;

use crate::MAX_INDEX_LEVEL;
pub(crate) const BLOCK_VEC_SIZE: usize = size_of::<BlockVec>();

#[derive(Clone, Copy)]
pub(crate) struct Block {
    pub next: *mut Block,
}
/// node index1 index2 ... levels
#[derive(Clone, Copy)]
pub(crate) struct BlockVec {
    pub(crate) index: [*mut Block; MAX_INDEX_LEVEL],
    pub(crate) levels: usize,
}

impl BlockVec {
    pub unsafe fn new(addr: usize, levels: usize) -> *mut Self {
        let vec = addr as *mut usize as *mut BlockVec;
        (*vec).index = [ptr::null_mut(); MAX_INDEX_LEVEL];
        (*vec).levels = levels;
        vec
    }

    pub unsafe fn from_addr(addr: usize) -> *mut Self {
        addr as *mut usize as *mut BlockVec
    }
}
