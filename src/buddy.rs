use core::alloc::Layout;

use crate::{
    block::BLOCK_VEC_SIZE, logarithmic_two_down, logarithmic_two_up, page_round_up, SkipList,
};
use lego_spec::memory::{AllocError, PhysicalPageAllocator};

pub struct BuddyAllocator<const BUDDY_POWER_NUM: usize> {
    free_lists: [SkipList; BUDDY_POWER_NUM],
    // 最小页尺寸
    page_size: usize,
    // 内存开始地址
    start_addr: usize,
    // SkipList 头部之后的第一块内存地址
    free_start: usize,
    // 系统总内存大小
    total_size: usize,
    // 可分配内存大小
    free_size: usize,
}

impl<const BUDDY_POWER_NUM: usize> BuddyAllocator<BUDDY_POWER_NUM> {
    /// # Safety
    pub const fn new(start_addr: usize, end_addr: usize, page_size: usize) -> Self {
        assert!(start_addr % page_size == 0 && end_addr % page_size == 0);
        Self {
            free_lists: [const { unsafe { SkipList::new() } }; BUDDY_POWER_NUM],
            start_addr,
            page_size,
            total_size: end_addr - start_addr,
            free_size: 0,
            free_start: 0,
        }
    }

    /// # Safety
    pub unsafe fn init(&mut self, kernel_end: usize) {
        let lists_heads_size = BLOCK_VEC_SIZE * BUDDY_POWER_NUM;
        self.free_start = page_round_up(kernel_end + lists_heads_size, self.page_size);

        let end_addr = self.start_addr + self.total_size;
        assert!(kernel_end > self.start_addr && self.free_start < end_addr);

        let size = end_addr - self.free_start;
        let min_power = self.page_size.trailing_zeros() as usize;
        self.free_lists
            .iter_mut()
            .enumerate()
            .for_each(|(index, list)| {
                let head_start = kernel_end + index * BLOCK_VEC_SIZE;
                let power = min_power + index;
                let block_size = 1 << power;
                let levels = logarithmic_two_down(size / block_size);
                list.init(head_start, block_size, levels);
            });

        let mut current_addr = self.free_start;
        let mut blk_idx = 0;
        while current_addr < end_addr {
            self.free_lists[0].insert(current_addr, blk_idx);
            current_addr += self.page_size;
            blk_idx += 1;
        }
        self.free_size = end_addr - self.free_start;
    }

    unsafe fn division_block(&mut self, addr: usize, index_low: usize, index_high: usize) {
        for index in (index_low..index_high).rev() {
            let block_size = self.free_lists[index].block_size;
            let address = addr + block_size;
            let block_index = self.calculate_block_index(address, block_size);
            self.free_lists[index].insert(address, block_index);
        }
    }

    #[inline]
    fn calculate_block_index(&self, addr: usize, block_size: usize) -> usize {
        (addr - self.free_start) / block_size
    }
}

unsafe impl<const BUDDY_POWER: usize> PhysicalPageAllocator for BuddyAllocator<BUDDY_POWER> {
    fn total_size(&self) -> usize {
        self.total_size
    }

    fn free_size(&self) -> usize {
        self.free_size
    }

    unsafe fn alloc_pages(&mut self, layout: Layout) -> Result<*mut u8, AllocError> {
        let ly = match layout.align_to(self.page_size) {
            Ok(ly) => ly,
            Err(_) => return Err(AllocError::Misaligned(layout)),
        };
        let power = logarithmic_two_up(ly.pad_to_align().size());
        let size = 1 << power;
        let pos = power - self.page_size.trailing_zeros() as usize;
        if let Some(ptr) = self.free_lists[pos].pop() {
            return Ok(ptr);
        }
        let calculate_blk_idx = |addr, block_size| (addr - self.free_start) / block_size;
        for index in (0..pos).rev() {
            if let Some(ptr) =
                self.free_lists[index].remove_continuous_space(size, calculate_blk_idx)
            {
                return Ok(ptr);
            }
        }
        for index in (pos + 1)..self.free_lists.len() {
            if let Some(ptr) = self.free_lists[index].pop() {
                self.division_block(ptr as usize, pos, index);
                return Ok(ptr);
            }
        }
        Err(AllocError::OutOfMemory(layout))
    }

    unsafe fn free_pages(&mut self, ptr: *mut u8, layout: Layout) -> Result<(), AllocError> {
        let addr = ptr as usize;
        if ptr.is_null() {
            return Err(AllocError::NullPointer(addr));
        }
        if addr < self.free_start || addr >= self.start_addr + self.total_size {
            return Err(AllocError::IllegalAddr(addr));
        }
        let offset = ptr as usize - self.start_addr;
        let ly = match layout.align_to(self.page_size) {
            Ok(ly) => ly,
            Err(_) => return Err(AllocError::Misaligned(layout)),
        };
        let power = logarithmic_two_up(ly.pad_to_align().size());
        let size = 1 << power;
        if offset % size != 0 {
            return Err(AllocError::Misaligned(layout));
        }
        let pos = power - self.page_size.trailing_zeros() as usize;
        if pos <= self.free_lists.len() {
            self.free_lists[pos].insert(addr, offset / size);
        } else {
            let max_block_size = self.free_lists[self.free_lists.len() - 1].block_size;
            let mut current = addr;
            while current < addr + size {
                let block_index = self.calculate_block_index(addr, max_block_size);
                self.free_lists[self.free_lists.len() - 1].insert(current, block_index);
                current += max_block_size;
            }
        }
        self.free_size += size;
        Ok(())
    }

    fn allocated_size(&self) -> usize {
        self.total_size - self.free_size
    }
}
