use crate::LogError;
use crate::memory::*;
use crate::memory::address::*;
use crate::memory::map;
use crate::memory::paging::*;
use crate::memory::paging::tracing::*;
use crate::memory::paging::tracing::management::{
    TRACE_PAGE_OWNER, TRACE_PAGE_TYPE
};


const
SIZE_THRESHOLD : usize = METADATA_ARRAY_SIZE / 100 * 70;


/// Removes the unused [`TracingPage`]s
///
/// A [`TracingPage`] is considered unused if it stores no allocations.
/// Up to one unused page is left allocated in case the preceding page
/// is filled by more than 70%, or if it is the only page.
///
/// ## Warning
///
/// This function assumes that the content of the pages has been merged
/// already. Violating this pre-condition may cause serious disruption.
/// Use with care.
///
/// ## Note
///
/// Only pages are removed, not tables.
///
/// ## Returns
///
/// Returns an 'Err` if deallocating a page failed, otherwise returns an
/// empty [`Ok`].
pub(in crate::memory)
fn cleanup_unused_tracing_pages(
    owner:MemoryOwner,
) -> Result<(), PagingError> {
    let mut delete_next = false;
    for entry in TracingPagesIterator::new(owner) {
        if delete_next {
            let page_paddr = PhysicalAddress::from(entry.bitmap().address(TRACE_PAGE_TYPE));
            map::release_range(page_paddr, TRACE_PAGE_TYPE, 1, TRACE_PAGE_OWNER)
                .log_map_err(PagingError::DeallocationFailure)?;
            entry.set_bitmap(Bitmap::new());
        } else {
            let tracing_page = unsafe { &*TracingPage::from_table_entry(entry) };
            if tracing_page.size() < SIZE_THRESHOLD {
                delete_next |= true;
            }
        }
    }
    Ok(())
}
