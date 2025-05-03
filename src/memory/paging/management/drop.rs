use crate::memory::address::*;
use crate::memory::map;
use crate::memory::paging::*;


/// Drops the space previously taken by an allocation, marking the corresponding
/// memory as available
///
/// ## Returns
///
/// Returns an empty [`Ok`] if no error occured, otherwise returns an [`Err`]
/// containing the error.
pub(in crate::memory)
fn drop_occupied_space(
    laddr:LogicalAddress,
    owner:MemoryOwner,
) -> Result<(),PagingError> {
    update_tracing_table(laddr, owner)
        .and_then(|(paddr,size)| update_memory_map(paddr, size, owner))
}

fn update_memory_map(
    paddr:PhysicalAddress,
    size:u64,
    owner:MemoryOwner,
) -> Result<(), PagingError> {
    map::drop_space_unconstrained(paddr, size, owner)
        .or(Err(PagingError::InternalFailure))
}

fn update_tracing_table(
    laddr:LogicalAddress,
    owner:MemoryOwner,
) -> Result<(PhysicalAddress, u64),PagingError> {
    tracing::drop::drop_occupied_space(laddr, owner)
}
