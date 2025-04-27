#![allow(non_upper_case_globals)]

use super::*;


const SIZE_512KiB : u64 = SIZE_2MiB / 4;
const SIZE_256MiB : u64 = SIZE_1GiB / 4;


/// Computes the type and number of pages to be used to allocate the
/// requested size
pub(in crate::memory)
fn suitable_pages(target_size:u64) -> (PageType,u64) {
    let page_type = if target_size < SIZE_512KiB {
        PageType::FourKiB
    } else if target_size < SIZE_256MiB {
        PageType::TwoMiB
    } else {
        PageType::OneGiB
    };
    let page_size : u64 = page_type.into();
    let n_pages = match (target_size/page_size, target_size%page_size) {
        (n,0) => n,
        (n,_) => n+1,
    };
    (page_type,n_pages)
}
