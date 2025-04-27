use crate::LogError;
use crate::memory::address::*;
use crate::memory::map;
use crate::memory::paging::*;


/// Takes the space requested by an allocation, marking the corresponding
/// memory as occupied
///
/// ## Returns
///
/// Returns an empty [`Ok`] if no error occured, otherwise returns an [`Err`]
/// containing the error.
pub(in crate::memory)
fn take_available_space(
    paddr:PhysicalAddress,
    size:u64,
    owner:MemoryOwner,
) -> Result<(),PagingError> {
    update_memory_map(paddr, size, owner)?;
    if let Err(e) = update_tracing_table(paddr, size, owner) {
        revert_memory_map_update(paddr, size, owner);
        return Err(e);
    }
    Ok(())
}

fn update_memory_map(
    paddr:PhysicalAddress,
    size:u64,
    owner:MemoryOwner,
) -> Result<(), PagingError> {
    map::take_space_unconstrained(paddr, size, owner)
        .log_map_err(PagingError::InternalFailure)
}

fn revert_memory_map_update(
    paddr:PhysicalAddress,
    size:u64,
    owner:MemoryOwner,
) {
    let _ = map::drop_space_unconstrained(paddr, size, owner).log_err();
}

fn update_tracing_table(
    paddr:PhysicalAddress,
    size:u64,
    owner:MemoryOwner,
) -> Result<(),PagingError> {
    tracing::take::take_available_space(paddr, size, owner)
}
