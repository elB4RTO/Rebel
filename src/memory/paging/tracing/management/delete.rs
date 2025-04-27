use crate::LogError;
use crate::memory::*;
use crate::memory::address::*;
use crate::memory::map;
use crate::memory::paging::*;
use crate::memory::paging::tracing::*;
use crate::memory::paging::tracing::management::{
    TRACE_PAGE_OWNER, TRACE_PAGE_TYPE
};


/// Removes the [`TracingPage`] allocated at the given [`PhysicalAddress`]
/// and de-allocates it
///
/// ## Returns
///
/// Returns an [`Err`] if the page cannot be found or if de-allocating failed,
/// otherwise returns an empty [`Ok`].
pub(in crate::memory)
fn delete_tracing_page(
    page_paddr:PhysicalAddress,
    owner:MemoryOwner,
) -> Result<(), PagingError> {
    for entry in TracingPagesIterator::new(owner) {
        let paddr = PhysicalAddress::from(entry.bitmap().address(TRACE_PAGE_TYPE));
        if page_paddr == paddr {
            map::release_range(page_paddr, TRACE_PAGE_TYPE, 1, TRACE_PAGE_OWNER)
                .log_map_err(PagingError::DeallocationFailure)?;
            entry.set_bitmap(Bitmap::new());
            return Ok(());
        }
    }
    Err(PagingError::PageFault)
}
