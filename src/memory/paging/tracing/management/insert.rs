use crate::memory::MemoryOwner;
use crate::memory::address::*;
use crate::memory::paging::PagingError;
use crate::memory::paging::tracing::*;


/// Inserts the given amount of free space in the corresponding [`TracingPage`]
///
/// Simply calls [`insert_entry()`] with a [`Metadata`] of free type
pub(in crate::memory::paging)
fn insert_available_space(
    paddr:PhysicalAddress,
    laddr:LogicalAddress,
    size:u64,
    owner:MemoryOwner,
) -> Result<(),PagingError> {
    insert_entry(Metadata::new_free(paddr, laddr, size), owner)
}


/// Inserts the given [`Metadata`] in the corresponding [`TracingPage`]
///
/// ## Returns
///
/// Returns an empty [`Ok`] if successfull, otherwise returns an [`Err`]
/// with the error.
pub(in crate::memory::paging::tracing)
fn insert_entry(
    md:Metadata,
    owner:MemoryOwner
) -> Result<(), PagingError> {
    let mut found = false;
    let mut excess = Excess::default();
    for entry in TracingPagesIterator::new(owner) {
        let tracing_page = unsafe { &mut *TracingPage::from_table_entry(entry) };
        if let Some(md) = excess.pop() {
            excess = tracing_page.try_push(md)
                .map(|o| o.into())
                .map_err(|e| PagingError::from(e))?;
        } else if tracing_page.is_empty() || tracing_page.lower_paddr() > md.lower_paddr() || tracing_page.contains_paddr(md.lower_paddr()) {
            found |= true;
            excess = tracing_page.insert(md)
                .map_err(|e| PagingError::from(e))?;
        } else {
            continue;
        }
        if excess.is_none() {
            return Ok(());
        }
    }
    if !found {
        return Err(PagingError::from(TracingError::NotFound));
    }
    if let Some(md) = excess.pop() {
        let tracing_table_entry = create::create_tracing_page(owner)?;
        let tracing_page = unsafe { &mut *TracingPage::from_table_entry(tracing_table_entry) };
        tracing_page.append_unchecked(md);
        if !excess.is_none() {
            return Err(PagingError::from(TracingError::InternalFailure));
        }
    }
    Ok(())
}
