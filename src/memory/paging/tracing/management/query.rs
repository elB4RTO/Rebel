use crate::memory::MemoryOwner;
use crate::memory::address::*;
use crate::memory::paging::PagingError;
use crate::memory::paging::tracing::*;


/// Checks whether the allocation pointed to by the given [`LogicalAddress`]
/// can be relocated in-place by shrinking or extending its size to match
/// the given `size`
///
/// ## Returns
///
/// Returns an [`Err`] if the given [`LogicalAddress`] cannot be found
/// in the tracing tables. Otherwise returns an [`Ok`] with the result:
/// if it's possible to relocate in-place, the first element is set
/// to `true` and the second element to `0`, otherwise the first element
/// is set to `false` and the second element is set to the size of
/// the current allocation.
pub(in crate::memory::paging)
fn can_relocate_inplace(
    laddr:LogicalAddress,
    size:u64,
    owner:MemoryOwner,
) -> Result<(bool,u64),PagingError> {
    for entry in TracingPagesIterator::new(owner) {
        let tracing_page = unsafe { &mut *TracingPage::from_table_entry(entry) };
        let mut this_one = false;
        let mut allocated_size = 0_u64;
        for md_ptr in tracing_page.iterate() {
            let md = unsafe { &*md_ptr };
            if this_one {
                match md.is_free() && md.size() >= size-allocated_size {
                    true  => return Ok((true,0)),
                    false => return Ok((false,allocated_size)),
                }
            } else if md.lower_laddr() == laddr {
                if md.is_free() {
                    return Err(PagingError::from(TracingError::InvalidRequest));
                }
                allocated_size = md.size();
                if allocated_size >= size {
                    return Ok((true,0));
                }
                this_one |= true;
            }
        }
    }
    Err(PagingError::from(TracingError::NotFound))
}
