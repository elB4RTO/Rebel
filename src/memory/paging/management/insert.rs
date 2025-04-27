use crate::log::*;
use crate::memory::address::*;
use crate::memory::memset;
use crate::memory::paging::*;
use crate::memory::paging::iterators::PageTableEntryOffset;
use crate::memory::paging::iterators::allocations::*;
use crate::memory::map;

use core::ops::Add;


/// Creates a new page table of the given type
///
/// The new table is only inserted if a compatible parent table
/// already exists that can store it
///
/// ## Returns
///
/// Returns [`Err`] if there is no compatible table to store the table in.
pub(in crate::memory::paging)
fn insert_table(
    table_type:PageTableType,
    flags:Bitmap,
    owner:MemoryOwner,
) -> Result<Option<AllocationsIterator>, PagingError> {
    use PageTableType::*;
    let table_paddr = map::find_available(PageType::FourKiB)
        .ok_or(PagingError::NoSpaceAvailable)?;
    let bitmap = Bitmap::from(table_paddr) | flags;
    let table_iter_opt = match table_type {
        PageMapLevel4Table => return Err(PagingError::IncompatibleTable),
        PageDirectoryPointerTable => insert_table_pdpt(bitmap, owner),
        PageDirectoryTable => insert_table_pdt(bitmap, owner),
        PageTable => insert_table_pt(bitmap, owner),
    };
    match table_iter_opt {
        None => Ok(None),
        Some(table_iter) => {
            unsafe { memset(table_paddr.get(), 0, SIZE_4KiB); }
            map::acquire(table_paddr, PageType::FourKiB, owner)
                .log_map_err(PagingError::AllocationFailure)
                .map(|()| Some(table_iter))
        },
    }
}

fn insert_table_pdpt(
    bitmap:Bitmap,
    owner:MemoryOwner,
) -> Option<AllocationsIterator> {
    if let Some(mut pml4t_iterator) = AllocationsPml4tIterator::new(owner) {
        while let Some((_,pml4t_entry)) = pml4t_iterator.next() {
            if pml4t_entry.bitmap().present() {
                continue;
            }
            pml4t_entry.set_bitmap(bitmap);
            // NOTE:
            //   The PML4T entry bitmap must be written to memory before the new PDPT
            //   iterator is created, otherwise it will fail
            let parent_offsets = pml4t_iterator.offsets();
            let pdpt_offset = PageTableEntryOffset::allocations(PageTableType::PageDirectoryPointerTable, owner);
            return AllocationsIterator::from_offsets(parent_offsets.add(pdpt_offset), owner);
        }
    }
    None
}

fn insert_table_pdt(
    bitmap:Bitmap,
    owner:MemoryOwner,
) -> Option<AllocationsIterator> {
    if let Some(mut pdpt_iterator) = AllocationsPdptIterator::new(owner) {
        while let Some((_,pdpt_entry)) = pdpt_iterator.next() {
            if pdpt_entry.bitmap().present() {
                continue;
            }
            pdpt_entry.set_bitmap(bitmap);
            // NOTE:
            //   The PDPT entry bitmap must be written to memory before the new PDT
            //   iterator is created, otherwise it will fail
            let parent_offsets = pdpt_iterator.offsets();
            let pdt_offset = PageTableEntryOffset::allocations(PageTableType::PageDirectoryTable, owner);
            return AllocationsIterator::from_offsets(parent_offsets.add(pdt_offset), owner);
        }
    }
    None
}

fn insert_table_pt(
    bitmap:Bitmap,
    owner:MemoryOwner,
) -> Option<AllocationsIterator> {
    if let Some(mut pdt_iterator) = AllocationsPdtIterator::new(owner) {
        while let Some((_,pdt_entry)) = pdt_iterator.next() {
            if pdt_entry.bitmap().present() {
                continue;
            }
            pdt_entry.set_bitmap(bitmap);
            // NOTE:
            //   The PDT entry bitmap must be written to memory before the new PT
            //   iterator is created, otherwise it will fail
            let parent_offsets = pdt_iterator.offsets();
            let pt_offset = PageTableEntryOffset::allocations(PageTableType::PageTable, owner);
            return AllocationsIterator::from_offsets(parent_offsets.add(pt_offset), owner);
        }
    }
    None
}

fn insert_table_into(
    mut iterator:AllocationsIterator,
    table_type:PageTableType,
    flags:Bitmap,
    owner:MemoryOwner,
) -> Result<Option<AllocationsIterator>, PagingError> {
    if let Some((_,parent_table_entry)) = iterator.next() {
        if parent_table_entry.bitmap().present() {
            return Err(PagingError::InternalFailure);
        }
        let table_paddr = map::find_available(PageType::FourKiB)
            .ok_or(PagingError::NoSpaceAvailable)?;
        parent_table_entry.set_bitmap(Bitmap::from(table_paddr) | flags);
        // NOTE:
        //   The parent table's entry bitmap must be written to memory before the new
        //   table's iterator is created, otherwise it will fail
        let parent_offsets = iterator.offsets();
        let table_offset = PageTableEntryOffset::allocations(table_type, owner);
        if let Some(table_iter) = AllocationsIterator::from_offsets(parent_offsets.add(table_offset), owner) {
            unsafe { memset(table_paddr.get(), 0, SIZE_4KiB); }
            return map::acquire(table_paddr, PageType::FourKiB, owner)
                .log_map_err(PagingError::AllocationFailure)
                .map(|()| Some(table_iter));
        }
    }
    Err(PagingError::InternalFailure)
}

/// Creates tables recursively if they do not exist
///
/// Tries to create a table of the given type, if there is no parent table
/// that can store it as entry, calls itself to crate the parent table.
/// Stops trying to create parent tables when the parent table is a PML4T
fn recursively_insert_parent_table(
    parent_table_type:PageTableType,
    flags:Bitmap,
    owner:MemoryOwner,
) -> Result<Option<AllocationsIterator>, PagingError> {
    if parent_table_type == PageTableType::PageMapLevel4Table {
        return Ok(None);
    }
    if let Some(table_iter) = insert_table(parent_table_type, flags, owner)? {
        return Ok(Some(table_iter));
    }
    let grandparent_table_type = parent_table_type.parent_table_type_or_panic();
    match recursively_insert_parent_table(grandparent_table_type, flags, owner)? {
        Some(grandparent_table_iter) => insert_table_into(grandparent_table_iter, parent_table_type, flags, owner),
        None => Ok(None),
    }
    // NOTE:
    //   Need to handle deleting the new tables on failure???
    //   A maximum of 3 new tables will be allocated by a call to this function,
    //   for a total of 12 KiB of memory space.
    //   This function will be called when there is no other slot available and
    //   hence it is likely that it will be called again in the near future if
    //   these new tables get deleted at this point.
}



/// Attempts to create one page of the given [`PageType`]
///
/// Simply calls [`insert_pages()`] with `n_pages` set to `1`
pub(in crate::memory)
fn insert_page(
    page_type:PageType,
    flags:Bitmap,
    owner:MemoryOwner,
) -> Result<Option<TotalAddress>, PagingError> {
    insert_pages(page_type, 1, flags, owner)
}

/// Attempts to create `n` new pages of the given [`PageType`]
///
/// All the pages will be contiguous in memory.
///
/// ## Warning
///
/// This function does not set the _PageSize_ bit on the given flags, it
/// must be already set in case it's needed.
///
/// ## Returns
///
/// Returns [`Err`] if an error occured while allocating the page, otherwise
/// returns an [`Ok`] containing a [`Some`] with the base address of the
/// newly allocated pages, or containing a [`None`] if there are no available
/// table entries to insert the pages into.
pub(in crate::memory)
fn insert_pages(
    page_type:PageType,
    n_pages:u64,
    flags:Bitmap,
    owner:MemoryOwner,
) -> Result<Option<TotalAddress>, PagingError> {
    if let Some(iter) = search::find_pages_slots(page_type, n_pages, owner) {
        return insert_pages_into(iter, page_type, n_pages, flags, owner)
            .map(|addr| Some(addr));
    }
    Ok(None)
}

/// Attempts to create the given number of new pages of the given type,
/// in contiguous slots of the same table. If there is no slot series
/// available for them, tries to create a new page table. If there is
/// no slot available for the table, tries to create a parent table,
/// and so on.
///
/// ## Warning
///
/// This function does not set the _PageSize_ bit on the given flags, it
/// must be set already in case it's needed.
///
/// ## Returns
///
/// Returns the base address of the newly allocated pages inside an [`Ok`]
/// if successful.
/// Returns [`Err`] if all the possible slots are taken and it's not
/// possible to create new tables, or in case a memory error occured
/// during the allocation.
pub(in crate::memory)
fn force_insert_pages(
    page_type:PageType,
    n_pages:u64,
    flags:Bitmap,
    owner:MemoryOwner,
) -> Result<TotalAddress, PagingError> {
    if let Some(base_addr) = insert_pages(page_type, n_pages, flags, owner)? {
        return Ok(base_addr);
    }
    let parent_flags = flags.without_bits(PS_BIT);
    let parent_table_type = PageTableType::from(page_type);
    if let Some(parent_table_iter) = recursively_insert_parent_table(parent_table_type, parent_flags, owner)? {
        return insert_pages_into(parent_table_iter, page_type, n_pages, flags, owner);
    }
    Err(PagingError::NoSlotAvailable)
}

/// Inserts `n_pages` of `page_type` inside `table` starting from `start_idx`
fn insert_pages_into(
    mut iterator:AllocationsIterator,
    page_type:PageType,
    n_pages:u64,
    flags:Bitmap,
    owner:MemoryOwner,
) -> Result<TotalAddress, PagingError> {
    let page_size = page_type.into();
    if let Some(base_paddr) = map::find_available_range(page_type, n_pages) {
        if let Err(_) = map::acquire_range(base_paddr, page_type, n_pages, owner).log_err() {
            return Err(PagingError::AllocationFailure);
        }
        let base_laddr = iterator.build_laddr(LogicalAddress::from(0));
        let mut page_paddr = base_paddr;
        let mut count = 0_u64;
        while let Some((contiguous,entry)) = iterator.next() {
            if !contiguous {
                return Err(PagingError::InternalFailure);
            }
            entry.set_bitmap(flags.with_bits(page_paddr.get()));
            page_paddr += page_size;
            count += 1;
            if count == n_pages {
                break;
            }
        }
        if count < n_pages {
            return Err(PagingError::InternalFailure);
        }
        update_tracing_table(base_paddr, base_laddr, page_type, n_pages, owner)?;
        Ok(TotalAddress::new(base_paddr, base_laddr))
    } else {
        Err(PagingError::NoSpaceAvailable)
    }
}



/// Inserts the new pages in the tracing table
fn update_tracing_table(
    page_paddr:PhysicalAddress,
    page_laddr:LogicalAddress,
    page_type:PageType,
    n_pages:u64,
    owner:MemoryOwner,
) -> Result<(),PagingError> {
    let page_size : u64 = page_type.into();
    let total_size = page_size * n_pages;
    tracing::insert::insert_available_space(page_paddr, page_laddr, total_size, owner)
}
