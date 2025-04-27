use crate::memory::address::*;
use crate::memory::paging::*;


/// Checks whether the allocation pointed to by the given [`LogicalAddress`]
/// can be relocated in-place, by shrinking or extending its size to the
/// given `size`
///
/// ## Returns
///
/// Returns an [`Err`] if the given [`LogicalAddress`] cannot be found
/// in the tracing tables. Otherwise returns an [`Ok`] with the result:
/// if it's possible to relocate in-place, the first element is set
/// to `true` and the second element is `0`, otherwise the first element
/// is set to `false` and the second element is set to the size of
/// the current allocation.
pub(in crate::memory)
fn can_relocate_inplace(
    laddr:LogicalAddress,
    size:u64,
    owner:MemoryOwner,
) -> Result<(bool,u64),PagingError> {
    tracing::query::can_relocate_inplace(laddr, size, owner)
}
