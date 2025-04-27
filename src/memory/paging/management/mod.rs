pub(in crate::memory) mod clean;
pub(in crate::memory) mod create;
pub(in crate::memory) mod delete;
pub(in crate::memory) mod drop;
pub(in crate::memory) mod insert;
pub(in crate::memory) mod query;
pub(in crate::memory) mod remove;
pub(in crate::memory) mod search;
pub(in crate::memory) mod take;
pub(in crate::memory) mod update;

use crate::memory::address::*;


pub(crate)
fn switch_page_map(pml4t_addr:PhysicalAddress) {
    unsafe {
        crate::cpu::registers::set_cr3(pml4t_addr.get());
    }
}
