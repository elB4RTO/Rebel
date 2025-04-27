use crate::memory::MemoryOwner;
use crate::memory::address::*;
use crate::memory::paging::tracing::*;


/// Finds the logical and physical addresses of an entry that has enough
/// free space to store the requested size of memory
///
/// ## Returns
///
/// Returns [`None`] if there is currently no page with enough space,
/// otherwise return a [`Some`] containing the addresses.
pub(in crate::memory::paging)
fn find_available_space(size:u64, owner:MemoryOwner) -> Option<TotalAddress> {
    for entry in TracingPagesIterator::new(owner) {
        let tracing_page = TracingPage::from_table_entry(entry);
        for metadata in unsafe { (*tracing_page).iterate() } {
            let metadata = unsafe { metadata.as_ref().unwrap() };
            match metadata.is_free() {
                true => if metadata.size() >= size {
                    return Some(TotalAddress::new(metadata.lower_paddr(), metadata.lower_laddr()));
                },
                _ => continue,
            }
        }
    }
    None
}
