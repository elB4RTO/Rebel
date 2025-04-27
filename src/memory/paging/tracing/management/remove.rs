use crate::memory::MemoryOwner;
use crate::memory::address::*;
use crate::memory::paging::PagingError;
use crate::memory::paging::tracing::*;


/// Removes the given amount of space from the [`TracingPage`]s
///
/// ## Returns
///
/// Returns an [`Err`] if the given [`PhysicalAddress`] cannot be found in any
/// of the tables or if removing fails, otherwise returns an empty [`Ok`].
pub(in crate::memory::paging)
fn remove_space(
    paddr:PhysicalAddress,
    size:u64,
    owner:MemoryOwner,
) -> Result<(), PagingError> {
    let mut found = false;
    let mut reminder = Reminder::Zero;
    for entry in TracingPagesIterator::new(owner) {
        let tracing_page = unsafe { &mut *TracingPage::from_table_entry(entry) };
        match reminder {
            Reminder::Zero => {
                if tracing_page.contains_paddr_strict(paddr) {
                    found |= true;
                    reminder = tracing_page.try_remove(paddr, size)
                        .map_err(|e| PagingError::from(e))?;
                    if reminder.is_zero() {
                        return merge::merge_tracing_pages(owner);
                    }
                }
            },
            Reminder::Negative(md) => {
                // can only reach this point at the second iteration and only in case the removed space
                // was at the upper side of the last entry of the page, and only if the page is full and
                // the space to remove was bigger then the upper part of the entry itself.
                reminder = tracing_page.try_remove(md.lower_paddr(), md.size())?;
                if reminder.is_zero() {
                    return merge::merge_tracing_pages(owner);
                }
            },
            Reminder::Positive(md) => {
                // can only reach this point at the second iteration and only in case the removed space
                // was in the middle of an entry. in all other cases the page con only shrink or remain
                // the same, but cannot grow.
                reminder = match tracing_page.try_push(md)? {
                    Some(md) => Reminder::Positive(md),
                    None => return Ok(()),
                };
            }
        }
    }
    if !found {
        return Err(PagingError::from(TracingError::NotFound));
    }
    match reminder {
        Reminder::Positive(md) => {
            let tracing_table_entry = create::create_tracing_page(owner)?;
            let tracing_page = unsafe { &mut *TracingPage::from_table_entry(tracing_table_entry) };
            tracing_page.append_unchecked(md);
            Ok(())
        },
        Reminder::Negative(_) => Err(PagingError::from(TracingError::InvalidRequest)),
        Reminder::Zero => Err(PagingError::from(TracingError::InternalFailure)),
    }
}
