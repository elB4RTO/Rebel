use crate::memory::MemoryOwner;
use crate::memory::address::*;
use crate::memory::paging::PagingError;
use crate::memory::paging::tracing::*;


/// Drops the space previously taken by an allocation, marking the corresponding
/// entries as available
///
/// ## Returns
///
/// Returns an [`Err`] if the given [`LogicalAddress`] is not valid, if it doesn't
/// belong to any [`TracingPage`], if dropping fails, or another error occurs.
/// Otherwise returns an [`Ok`] containing the [`PhysicalAddress`] which translates
/// to the given [`LogicalAddress`].
pub(in crate::memory::paging)
fn drop_occupied_space(
    laddr:LogicalAddress,
    size:u64,
    owner:MemoryOwner,
) -> Result<PhysicalAddress, PagingError> {
    let paddr = laddr.to_physical(owner)
        .map_err(|e| PagingError::AddressError(e))?;
    for entry in TracingPagesIterator::new(owner) {
        let tracing_page = unsafe { &mut *TracingPage::from_table_entry(entry) };
        if tracing_page.contains_paddr_strict(paddr) {
            return tracing_page.drop(paddr, size)
                .map_err(|e| PagingError::from(e))
                .and_then(|()| merge::merge_tracing_pages(owner)
                    .map(|()| paddr));
        }
    }
    Err(PagingError::from(TracingError::NotFound))
}
