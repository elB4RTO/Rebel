use crate::memory::address::*;
use crate::memory::paging::*;
use crate::memory::paging::iterators::Duplicate;
use crate::memory::paging::iterators::allocations::*;


/// Finds the logical and physical addresses of an available location
/// in memory, mapped by an existing page, that has enough contiguous
/// space to store the given allocation size
///
/// ## Returns
///
/// Returns [`None`] if there is currently no page with enough space,
/// otherwise return a [`Some`] holding the addresses
pub(in crate::memory)
fn find_available_space(size:u64, owner:MemoryOwner) -> Option<TotalAddress> {
    tracing::search::find_available_space(size, owner)
}


/// Searches for a page table that can host one page of the given [`PageType`]
///
/// Simply calls [`find_pages_slots()`] with `n_pages` set to `1`.
pub(in crate::memory::paging)
fn find_page_slot(
    page_type:PageType,
    owner:MemoryOwner,
) -> Option<AllocationsIterator> {
    find_pages_slots(page_type, 1, owner)
}

/// Searches for a page table that can host the given number of pages of
/// the specified [`PageType`]
///
/// ## Returns
///
/// Returns [`None`] if there is currently no suitable table for the
/// given page type or none of the suitable tables has enough contiguous
/// free slots to host the given number of pages, otherwise returns [`Some`]
/// containing an iterator starting at the first available entry.
pub(in crate::memory::paging)
fn find_pages_slots(
    page_type:PageType,
    n_pages:u64,
    owner:MemoryOwner,
) -> Option<AllocationsIterator> {
    match page_type {
        PageType::OneGiB  => find_1gib_pages_slots(n_pages, owner),
        PageType::TwoMiB  => find_2mib_pages_slots(n_pages, owner),
        PageType::FourKiB => find_4kib_pages_slots(n_pages, owner),
    }
}

fn find_1gib_pages_slots(
    n_pages:u64,
    owner:MemoryOwner,
) -> Option<AllocationsIterator> {
    if let Some(mut pdpt_iterator) = AllocationsPdptIterator::new(owner) {
        let mut count = 0_u64;
        let mut iter = pdpt_iterator.duplicate();
        while let Some((contiguous,pdpt_entry)) = pdpt_iterator.next() {
            if !contiguous || pdpt_entry.bitmap().present() {
                iter = pdpt_iterator.duplicate_and_advance();
                count = 0;
                continue;
            }
            count += 1;
            if count == n_pages {
                return Some(iter.into());
            }
        }
    }
    None
}

fn find_2mib_pages_slots(
    n_pages:u64,
    owner:MemoryOwner,
) -> Option<AllocationsIterator> {
    if let Some(mut pdt_iterator) = AllocationsPdtIterator::new(owner) {
        let mut count = 0_u64;
        let mut iter = pdt_iterator.duplicate();
        while let Some((contiguous,pdt_entry)) = pdt_iterator.next() {
            if !contiguous || pdt_entry.bitmap().present() {
                iter = pdt_iterator.duplicate_and_advance();
                count = 0;
                continue;
            }
            count += 1;
            if count == n_pages {
                return Some(iter.into());
            }
        }
    }
    None
}

fn find_4kib_pages_slots(
    n_pages:u64,
    owner:MemoryOwner,
) -> Option<AllocationsIterator> {
    if let Some(mut pt_iterator) = AllocationsPtIterator::new(owner) {
        let mut count = 0_u64;
        let mut iter = pt_iterator.duplicate();
        while let Some((contiguous,pt_entry)) = pt_iterator.next() {
            if !contiguous || pt_entry.bitmap().present() {
                iter = pt_iterator.duplicate_and_advance();
                count = 0;
                continue;
            }
            count += 1;
            if count == n_pages {
                return Some(iter.into());
            }
        }
    }
    None
}
