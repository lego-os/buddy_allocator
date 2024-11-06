use crate::{block::BlockVec, MIN_INDEX_LEVEL};
use core::ptr;

pub(crate) struct SkipList {
    heads: *mut BlockVec,
    pub(crate) block_size: usize,
}

impl SkipList {
    pub const unsafe fn new() -> Self {
        Self {
            heads: ptr::null_mut(),
            block_size: 0,
        }
    }

    pub unsafe fn init(&mut self, heads_vec_addr: usize, block_size: usize, levels: usize) {
        self.heads = BlockVec::new(heads_vec_addr, levels);
        self.block_size = block_size;
    }

    pub unsafe fn insert(&mut self, addr: usize, block_index: usize) {
        let levels = (*self.heads).levels;
        let mut index_level = (block_index.trailing_zeros() & (usize::BITS - 1)) as usize;
        if index_level >= levels {
            index_level = levels - 1;
        }
        let new_block_vec = BlockVec::new(addr, index_level + 1);
        let mut current_addr = self.heads as usize;
        for index in (0..levels).rev() {
            let vec = BlockVec::from_addr(current_addr);
            let mut next_block = (*vec).get(index);
            while !next_block.is_null() && (next_block as usize) < addr {
                current_addr = next_block as usize & !(1 << (index + MIN_INDEX_LEVEL) - 1);
                next_block = (*next_block).next;
            }
            if index <= index_level {
                let new_block = (*new_block_vec).get(index);
                let current_block = (*vec).get(index);
                (*current_block).next = new_block;
                (*new_block).next = next_block;
            }
        }
    }

    pub unsafe fn pop(&mut self) -> Option<*mut u8> {
        let first = (*(*self.heads).get(0)).next;
        if first.is_null() {
            return None;
        }
        let block_vec = BlockVec::from_addr(first as usize);
        let levels = (*block_vec).levels;
        for index in 0..levels {
            let head = (*self.heads).get(index);
            let first_block = (*block_vec).get(index);
            (*head).next = (*first_block).next;
        }
        Some(first as *mut u8)
    }

    pub unsafe fn remove_continuous_space<F: Fn(usize, usize) -> usize>(
        &mut self,
        size: usize,
        calculate_blk_index: F,
    ) -> Option<*mut u8> {
        let mut res = None;
        let power = (size.trailing_zeros() - self.block_size.trailing_zeros()) as usize;
        let pre = (*self.heads).get(power);
        let mut current = (*pre).next;
        while current.is_null() {
            if calculate_blk_index(current as usize, self.block_size) % 2 == 0 {
                res = Some(current as *mut u8);
                (*pre).next = (*current).next;
                break;
            }
            current = (*current).next;
        }
        if res.is_some() {
            let current_addr = current as usize;
            let current_vec = BlockVec::from_addr(current_addr);
            let current_level = (*current_vec).levels - 1;
            let mut vec = self.heads;
            for index in (0..(*self.heads).levels).rev() {
                if index == power {
                    continue;
                }
                let mut p = (*vec).get(index);
                let mut n = (*p).next;
                while current_addr > n as usize {
                    p = n;
                    n = (*n).next;
                }
                if current_level <= index {
                    (*p).next = (*(*current_vec).get(index)).next;
                }
                vec = BlockVec::from_addr(pre as usize);
            }
        }
        res
    }
}
