use crate::memory::address::*;
use crate::memory::map;
use crate::memory::paging::*;
use crate::memory::paging::iterators::allocations::*;
use crate::memory::paging::iterators::indexers::*;


/// Removes one page of the given [`PageType`], which base address corresponds
/// to the given [`LogicalAddress`]
///
/// Simply calls [`remove_pages()`] with `n_pages` set to `1`
pub(in crate::memory)
fn remove_page(
    base_laddr:LogicalAddress,
    page_type:PageType,
    owner:MemoryOwner,
) -> Result<(),PagingError> {
    remove_pages(base_laddr, page_type, 1, owner)
}

/// Removes `n` pages of the given [`PageType`], which base address corresponds
/// to the given [`LogicalAddress`]
///
/// If `n_pages` is higher than `1`, all the pages shall be contiguous in memory
///
/// ## Warning
///
/// This function is not atomic, meaning that if an error occurs in a later stage
/// of processing, the pages removed so far cannot be restored. Use with extreme
/// care.
///
/// ## Returns
///
/// Returns an empty [`Ok`] if successfull, otherwise returns [`Err`] if the pages
/// are not contiguous, if any of the pages is not found, or if an error occurs
/// while deallocating.
pub(in crate::memory)
fn remove_pages(
    base_laddr:LogicalAddress,
    page_type:PageType,
    n_pages:u64,
    owner:MemoryOwner,
) -> Result<(),PagingError> {
    let base_paddr = match page_type {
        PageType::OneGiB  => remove_1gib_pages(base_laddr, n_pages, owner),
        PageType::TwoMiB  => remove_2mib_pages(base_laddr, n_pages, owner),
        PageType::FourKiB => remove_4kib_pages(base_laddr, n_pages, owner),
    }?;
    update_memory_map(base_paddr, page_type, n_pages, owner)?;
    update_tracing_table(base_paddr, page_type, n_pages, owner)
}

fn remove_1gib_pages(
    base_laddr:LogicalAddress,
    n_pages:u64,
    owner:MemoryOwner,
) -> Result<PhysicalAddress,PagingError> {
    let mut count = 0_u64;
    let pml4t_idx = PageTableEntryIndex::allocations_with_index(base_laddr.pml4t_index(), PageTableType::PageMapLevel4Table, owner);
    let pdpt_idx = PageTableEntryIndex::allocations_with_index(base_laddr.pdpt_index(), PageTableType::PageDirectoryPointerTable, owner);
    if let Some(mut pdpt_iterator) = AllocationsPdptIterator::new_with_offsets(pml4t_idx.into(), pdpt_idx.into(), owner) {
        let mut base_paddr = None;
        while let Some((contiguous,pdpt_entry)) = pdpt_iterator.next() {
            if !contiguous {
                return Err(PagingError::InvalidRequest);
            }
            let pdpt_entry_bitmap = pdpt_entry.bitmap();
            if !pdpt_entry_bitmap.present() {
                return Err(PagingError::PageNotPresent);
            } else if !pdpt_entry_bitmap.page_size() {
                return Err(PagingError::PageNotHuge);
            }
            if let None = base_paddr {
                base_paddr = Some(pdpt_entry_bitmap.address(PageType::OneGiB).into());
            }
            pdpt_entry.set_bitmap(Bitmap::new());
            count += 1;
            if count == n_pages {
                return base_paddr.ok_or(PagingError::InternalFailure);
            }
        }
    }
    if count > 0 {
        return Err(PagingError::InvalidRequest);
    }
    Err(PagingError::PageFault)
}

fn remove_2mib_pages(
    base_laddr:LogicalAddress,
    n_pages:u64,
    owner:MemoryOwner,
) -> Result<PhysicalAddress,PagingError> {
    let mut count = 0_u64;
    let pml4t_idx = PageTableEntryIndex::allocations_with_index(base_laddr.pml4t_index(), PageTableType::PageMapLevel4Table, owner);
    let pdpt_idx = PageTableEntryIndex::allocations_with_index(base_laddr.pdpt_index(), PageTableType::PageDirectoryPointerTable, owner);
    let pdt_idx = PageTableEntryIndex::allocations_with_index(base_laddr.pdt_index(), PageTableType::PageDirectoryTable, owner);
    if let Some(mut pdt_iterator) = AllocationsPdtIterator::new_with_offsets(pml4t_idx.into(), pdpt_idx.into(), pdt_idx.into(), owner) {
        let mut base_paddr = None;
        while let Some((contiguous,pdt_entry)) = pdt_iterator.next() {
            if !contiguous {
                return Err(PagingError::InvalidRequest);
            }
            let pdt_entry_bitmap = pdt_entry.bitmap();
            if !pdt_entry_bitmap.present() {
                return Err(PagingError::PageNotPresent);
            } else if !pdt_entry_bitmap.page_size() {
                return Err(PagingError::PageNotHuge);
            }
            if let None = base_paddr {
                base_paddr = Some(pdt_entry_bitmap.address(PageType::TwoMiB).into());
            }
            pdt_entry.set_bitmap(Bitmap::new());
            count += 1;
            if count == n_pages {
                return base_paddr.ok_or(PagingError::InternalFailure);
            }
        }
    }
    if count > 0 {
        return Err(PagingError::InvalidRequest);
    }
    Err(PagingError::PageFault)
}

fn remove_4kib_pages(
    base_laddr:LogicalAddress,
    n_pages:u64,
    owner:MemoryOwner,
) -> Result<PhysicalAddress,PagingError> {
    let mut count = 0_u64;
    let pml4t_idx = PageTableEntryIndex::allocations_with_index(base_laddr.pml4t_index(), PageTableType::PageMapLevel4Table, owner);
    let pdpt_idx = PageTableEntryIndex::allocations_with_index(base_laddr.pdpt_index(), PageTableType::PageDirectoryPointerTable, owner);
    let pdt_idx = PageTableEntryIndex::allocations_with_index(base_laddr.pdt_index(), PageTableType::PageDirectoryTable, owner);
    let pt_idx = PageTableEntryIndex::allocations_with_index(base_laddr.pt_index(), PageTableType::PageTable, owner);
    if let Some(mut pt_iterator) = AllocationsPtIterator::new_with_offsets(pml4t_idx.into(), pdpt_idx.into(), pdt_idx.into(), pt_idx.into(), owner) {
        let mut base_paddr = None;
        while let Some((contiguous,pt_entry)) = pt_iterator.next() {
            if !contiguous {
                return Err(PagingError::InvalidRequest);
            }
            let pt_entry_bitmap = pt_entry.bitmap();
            if !pt_entry_bitmap.present() {
                return Err(PagingError::PageNotPresent);
            }
            if let None = base_paddr {
                base_paddr = Some(pt_entry_bitmap.address(PageType::FourKiB).into());
            }
            pt_entry.set_bitmap(Bitmap::new());
            count += 1;
            if count == n_pages {
                return base_paddr.ok_or(PagingError::InternalFailure);
            }
        }
    }
    if count > 0 {
        return Err(PagingError::InvalidRequest);
    }
    Err(PagingError::PageFault)
}

fn update_memory_map(
    paddr:PhysicalAddress,
    page_type:PageType,
    n_pages:u64,
    owner:MemoryOwner,
) -> Result<(), PagingError> {
    map::release_range(paddr, page_type, n_pages, owner)
        .or(Err(PagingError::InternalFailure))
}

fn update_tracing_table(
    paddr:PhysicalAddress,
    page_type:PageType,
    n_pages:u64,
    owner:MemoryOwner,
) -> Result<(),PagingError> {
    let size = page_type.size() * n_pages;
    tracing::remove::remove_space(paddr, size, owner)
}
