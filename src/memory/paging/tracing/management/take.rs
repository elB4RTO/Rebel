use crate::memory::MemoryOwner;
use crate::memory::address::*;
use crate::memory::paging::PagingError;
use crate::memory::paging::tracing::*;


/// Takes the space requested by an allocation, marking the corresponding
/// entry as occupied
///
/// ## Returns
///
/// Returns an [`Err`] if the given [`PhysicalAddress`] cannot be found in any
/// of the tables or if taking fails, otherwise returns an empty [`Ok`].
pub(in crate::memory::paging)
fn take_available_space(
    paddr:PhysicalAddress,
    size:u64,
    owner:MemoryOwner,
) -> Result<(),PagingError> {
    let mut found = false;
    let mut excess = Excess::default();
    for entry in TracingPagesIterator::new(owner) {
        let tracing_page = unsafe { &mut *TracingPage::from_table_entry(entry) };
        if !excess.is_none() {
            excess = tracing_page.try_push_excess(excess)
                .map_err(|e| PagingError::from(e))?;
        } else if tracing_page.contains_paddr_strict(paddr) {
            found |= true;
            excess = tracing_page.take(paddr, size)
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
    if !excess.is_none() {
        let tracing_table_entry = create::create_tracing_page(owner)?;
        let tracing_page = unsafe { &mut *TracingPage::from_table_entry(tracing_table_entry) };
        while let Some(md) = excess.pop() {
            tracing_page.append_unchecked(md);
        }
    }
    Err(PagingError::InternalFailure)
}
