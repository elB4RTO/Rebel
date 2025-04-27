use crate::memory::*;
use crate::memory::paging::*;
use crate::memory::paging::tracing::*;


/// Merges the content of all the [`TracingPage`]s
///
/// If a page is found which is not completely full, the corresponding
/// number of entries are moved from the beginning of the next page (if
/// any) and appended to the current one.
/// This function only moves [`Metadata`] entries and does not remove
/// any [`TracingPage`], regardless of whether they're empty or not.
///
/// ## Note
///
/// This process is fundamental to ensure that tracing will behave as
/// expected. Leaving holes in the pages may lead to disaster.
///
/// ## Returns
///
/// Returns an [`Err`] containing the error if moving the entries fails,
/// otherwise returns an empty [`Ok`].
pub(in crate::memory)
fn merge_tracing_pages(
    owner:MemoryOwner,
) -> Result<(), PagingError> {
    let mut prev_page_ptr = core::ptr::null_mut::<TracingPage>();
    for entry in TracingPagesIterator::new(owner) {
        let curr_page_ptr = TracingPage::from_table_entry(entry);
        if !prev_page_ptr.is_null() {
            let prev_page = unsafe { &mut*prev_page_ptr };
            let curr_page = unsafe { &mut*curr_page_ptr };

            while !(prev_page.is_full() | curr_page.is_empty()) {
                if !prev_page.append(curr_page.extract(0)).is_none() {
                    return Err(PagingError::from(TracingError::InternalFailure));
                }
            }
        }
        prev_page_ptr = curr_page_ptr;
    }
    Ok(())
}
