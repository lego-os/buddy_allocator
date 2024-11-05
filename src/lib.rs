#![no_std]
mod buddy;
mod list;
mod block;
use list::*;

pub use buddy::BuddyAllocator;
const MAX_INDEX_LEVEL:usize = 30;
#[inline]
fn logarithmic_two_up(num: usize) -> usize {
    let trailing_zeros = num.trailing_zeros() as usize;
    if 1 << trailing_zeros == num {
        trailing_zeros
    } else {
        let start_zero = usize::BITS - num.leading_zeros();
        start_zero as usize
    }
}

#[inline]
fn logarithmic_two_down(num: usize) -> usize {
    let trailing_zeros = num.trailing_zeros() as usize;
    if 1 << trailing_zeros == num {
        trailing_zeros
    } else {
        let power_two = usize::BITS - num.leading_zeros() - 1;
        power_two as usize
    }
}

#[inline]
const fn page_round_up(addr: usize, page_size: usize) -> usize {
    let power = page_size.trailing_zeros();
    let res_addr = (addr >> power) << power;
    if res_addr == addr {
        addr
    } else {
        res_addr + page_size
    }
}

// #[inline]
// const fn page_round_down(addr: usize, page_size: usize)->usize{
//     let power = page_size.trailing_zeros();
//     let res_addr = (addr >> power) << power;
//     if res_addr == addr {
//         addr
//     } else {
//         res_addr - page_size
//     }
// }
