use crate::{Log,LogError};
use crate::memory::*;
use crate::memory::address::*;
use crate::memory::map;
use crate::memory::memset;
use crate::memory::paging::*;
use crate::memory::paging::iterators::PageTableEntryOffset;
use crate::memory::paging::tracing::*;
use crate::memory::paging::tracing::management::{
    TRACE_PAGE_FLAGS, TRACE_PAGE_OWNER, TRACE_PAGE_TYPE, TRACE_PAGE_SIZE
};


/// Allocates and initializes a new [`TracingPage`], appending it to the trace
/// table
///
/// ## Returns
///
/// Returns an [`Err`] if there's no memory to allocate the new page, if the
/// allocation failed or if there's no free slot in the trace table.
/// Otherwise returns an [`Ok`] containing a [`PageTableEntry`] pointing to
/// the newly created [`TracingPage`].
pub(in crate::memory)
fn create_tracing_page(
    owner:MemoryOwner,
) -> Result<PageTableEntry, PagingError> {
    let page_paddr = map::find_available_range(TRACE_PAGE_TYPE, 1)
        .ok_or(PagingError::NoSpaceAvailable)?;

    map::acquire_range(page_paddr, TRACE_PAGE_TYPE, 1, TRACE_PAGE_OWNER)
        .log_map_err(PagingError::AllocationFailure)?;

    unsafe {
        memset(page_paddr.get(), 0, TRACE_PAGE_SIZE);
    }

    append_tracing_page(page_paddr, owner)
        .map(|entry| {
            TracingPage::init(page_paddr);
            entry
        }).or_else(|e| {
            map::release_range(page_paddr, TRACE_PAGE_TYPE, 1, TRACE_PAGE_OWNER)
                .or_else(|em| {
                    e.log();
                    em.log();
                    Err(PagingError::DeallocationFailure)
                }).and(Err(e))
        })
}

/// Inserts the page allocated at the given `page_paddr` in the first
/// available entry of the first available PDT table reserved for tracing
///
/// If no PDT table is found with available entries, a new one is created.
///
/// ## Returns
///
/// Returns an [`Ok`] containing a [`PageTableEntry`] pointing to the new
/// [`TracingPage`] if successfull, otherwise returns [`Err`] with the error.
fn append_tracing_page(
    page_paddr:PhysicalAddress,
    owner:MemoryOwner,
) -> Result<PageTableEntry, PagingError> {
    for entry in TracingPDTEntriesIterator::new(owner) {
        if entry.bitmap().present() {
            continue;
        }
        entry.set_bitmap(Bitmap::from(page_paddr) | TRACE_PAGE_FLAGS);
        return Ok(entry);
    }
    // no more slots, need to allocate a new table
    let pdpt_entry = create_tracing_table(owner)?;
    let pdt_table = PageTable::try_from(pdpt_entry)?;
    let pdt_entry_offset = PageTableEntryOffset::tracing(PageTableType::PageDirectoryTable, owner)?;
    let pdt_entry = pdt_table.at_offset_unchecked(&pdt_entry_offset);
    pdt_entry.set_bitmap(Bitmap::from(page_paddr) | TRACE_PAGE_FLAGS);
    return Ok(pdt_entry);
}

/// Allocates a new PDT table in the first available entry of the PDPT table reserved
/// for tracing
///
/// ## Returns
///
/// Returns an [`Err`] if there's no memory available, if allocating failed, or if
/// there are no more available entries in the PDPT table. Otherwise returns an [`Ok`]
/// holding the new [`PageTableEntry`].
fn create_tracing_table(
    owner:MemoryOwner,
) -> Result<PageTableEntry, PagingError> {
    let table_paddr = map::find_available(PageType::FourKiB)
        .ok_or(PagingError::NoSpaceAvailable)?;

    map::acquire(table_paddr, PageType::FourKiB, TRACE_PAGE_OWNER)
        .log_map_err(PagingError::AllocationFailure)?;

    unsafe {
        memset(table_paddr.get(), 0, SIZE_4KiB);
    }

    let flags = Bitmap::default_kernel();

    for pdpt_entry in TracingPDPTEntriesIterator::new(owner) {
        if pdpt_entry.bitmap().present() {
            continue;
        }
        pdpt_entry.set_bitmap(Bitmap::from(table_paddr) | flags);
        return Ok(pdpt_entry);
    }
    Err(PagingError::NoSlotAvailable)
}
