use crate::memory::info;
use crate::memory::map;
use crate::memory::paging;


/// Initializes the structures responsible for the memory management
pub(crate)
fn init() {
    // initialize the kernel's basic pages structure (identity page, reserved pages and stack pages)
    paging::init();
    // initialize the memory map
    let total_size = info::memory_size();
    map::init(total_size);
    info::parse_smap();
}
