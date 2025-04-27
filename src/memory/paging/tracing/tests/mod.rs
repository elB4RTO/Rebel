mod page;
mod management;

use crate::memory::MemoryOwner;
use crate::memory::paging::tracing::*;

pub(crate)
fn run() {
    page::run_all_tests();
    management::run_all_tests();
}


#[derive(Default, Clone, Copy)]
struct TracingCount {
    pub(crate) n_free  : usize,
    pub(crate) n_taken : usize,
}

impl TracingCount {
    fn new(owner:MemoryOwner) -> Self {
        let mut tracing = Self::default();
        for entry in TracingPagesIterator::new(owner) {
            let tracing_page = unsafe { &mut *TracingPage::from_table_entry(entry) };
            for md_entry in tracing_page.iterate() {
                let metadata = unsafe { &*md_entry };
                if metadata.is_free() {
                    tracing.n_free += 1;
                } else if metadata.is_taken() {
                    tracing.n_taken += 1;
                }
            }
        }
        tracing
    }
}

pub(in crate::memory)
fn check_tracing(
    n_free:usize,
    n_taken:usize,
) -> bool {
    let tracing = TracingCount::new(MemoryOwner::Kernel);
    return n_free  == tracing.n_free
        && n_taken == tracing.n_taken;
}

pub(in crate::memory)
fn count_tracing_pages() -> usize {
    TracingPagesIterator::new(MemoryOwner::Kernel).count()
}

pub(in crate::memory::paging::tracing)
fn compare_metadata(lhs:&Metadata, rhs:&Metadata) -> bool {
    return lhs.lower_paddr() == rhs.lower_paddr()
        && lhs.lower_laddr() == rhs.lower_laddr()
        && lhs.size() == rhs.size()
        && lhs.is_free() == rhs.is_free()
        && lhs.is_taken() == rhs.is_taken()
        && lhs.is_none() == rhs.is_none();
}
