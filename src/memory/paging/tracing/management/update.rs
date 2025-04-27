use crate::memory::MemoryOwner;
use crate::memory::address::*;
use crate::memory::paging::PagingError;
use crate::memory::paging::tracing::*;



/// Resizes a taken entry, by shrinking or expanding it based on `size`
///
/// ## Returns
///
/// Returns an empty [`Ok`] if successfull, otherwise returns an [`Err`]
/// containing the error.
pub(in crate::memory::paging)
fn resize(
    paddr:PhysicalAddress,
    size:u64,
    owner:MemoryOwner,
) -> Result<(),PagingError> {
    let mut found = false;
    let mut excess = Excess::default();
    let mut rem_size = Reminder::Zero;
    for entry in TracingPagesIterator::new(owner) {
        let tracing_page = unsafe { &mut *TracingPage::from_table_entry(entry) };
        if let Some(md) = excess.pop() {
            // there's an exceeding metadata entry from the previous page that need
            // to be inserted in the current page
            excess = tracing_page.try_push(md)
                .map(|o| o.into())
                .map_err(|e| PagingError::from(e))?;
        } else if !rem_size.is_zero() {
            // there's an exceeding amount of memory from the previous page that need
            // to be added to or subtracted from the first entry of the current page
            use Reminder::*;
            match rem_size {
                Positive(md) => {
                    excess = tracing_page.try_push(md)
                        .map(|o| o.into())
                        .map_err(|e| PagingError::from(e))?;
                    if excess.is_none() {
                        return Ok(());
                    }
                    rem_size = Reminder::zero();
                },
                Negative(md) => {
                    tracing_page.take(md.lower_paddr(), md.size())
                        .map_err(|e| PagingError::from(e))?;
                    break;
                },
                Zero => return Err(PagingError::from(TracingError::InternalFailure)),
            }
        } else if tracing_page.contains_paddr(paddr) {
            found |= true;
            rem_size = tracing_page.resize(paddr, size)
                .map_err(|e| PagingError::from(e))?;
            if rem_size.is_zero() {
                return merge::merge_tracing_pages(owner);
            }
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
