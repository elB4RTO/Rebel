use crate::memory::address::*;
use crate::memory::map;
use crate::memory::paging::*;
use crate::memory::paging::iterators::PageTableEntryIndex;
use crate::memory::paging::tracing;

use core::ops::Add;


/// Resizes an allocation inplace, by shrinking or expanding it based on `size`
///
/// ## Returns
///
/// Returns an empty [`Ok`] if successfull, otherwise returns an [`Err`]
/// containing the error.
pub(in crate::memory)
fn relocate_inplace(
    laddr:LogicalAddress,
    size:u64,
    owner:MemoryOwner,
) -> Result<(),PagingError> {
    let pml4t = PageTable::pml4t(owner);
    let pml4t_index = PageTableEntryIndex::allocations_with_index(laddr.pml4t_index(), PageTableType::PageMapLevel4Table, owner);
    if let Some(pml4t_entry) = pml4t.at_index(&pml4t_index) {
        let pdpt = PageTable::try_from(pml4t_entry)?;
        let pdpt_index = PageTableEntryIndex::allocations_with_index(laddr.pdpt_index(), PageTableType::PageDirectoryPointerTable, owner);
        if let Some(pdpt_entry) = pdpt.at_index(&pdpt_index) {
            let pdpt_entry_bits = pdpt_entry.bitmap();
            if !pdpt_entry_bits.present() {
                return Err(PagingError::InvalidRequest);
            } else if pdpt_entry_bits.page_size() {
                let paddr = PhysicalAddress::from(pdpt_entry_bits.address(PageType::OneGiB))
                    .add(laddr.page_offset(PageType::OneGiB));
                return update_memory_map(paddr, size, owner)
                    .and_then(|()| update_tracing_table(paddr, size, owner));
            }
            let pdt = PageTable::try_from(pdpt_entry)?;
            let pdt_index = PageTableEntryIndex::allocations_with_index(laddr.pdt_index(), PageTableType::PageDirectoryTable, owner);
            if let Some(pdt_entry) = pdt.at_index(&pdt_index) {
                let pdt_entry_bits = pdt_entry.bitmap();
                if !pdt_entry_bits.present() {
                    return Err(PagingError::InvalidRequest);
                } else if pdt_entry_bits.page_size() {
                    let paddr = PhysicalAddress::from(pdpt_entry_bits.address(PageType::TwoMiB))
                        .add(laddr.page_offset(PageType::TwoMiB));
                    return update_memory_map(paddr, size, owner)
                        .and_then(|()| update_tracing_table(paddr, size, owner));
                }
                let pt = PageTable::try_from(pdt_entry)?;
                let pt_index = PageTableEntryIndex::allocations_with_index(laddr.pt_index(), PageTableType::PageTable, owner);
                if let Some(pt_entry) = pt.at_index(&pt_index) {
                    let pt_entry_bits = pt_entry.bitmap();
                    if !pt_entry_bits.present() {
                        return Err(PagingError::InvalidRequest);
                    }
                    let paddr = PhysicalAddress::from(pdpt_entry_bits.address(PageType::FourKiB))
                        .add(laddr.page_offset(PageType::FourKiB));
                    return update_memory_map(paddr, size, owner)
                        .and_then(|()| update_tracing_table(paddr, size, owner));
                }
            }
        }
    }
    Err(PagingError::PageFault)
}

fn update_memory_map(
    paddr:PhysicalAddress,
    size:u64,
    owner:MemoryOwner,
) -> Result<(), PagingError> {
    map::take_space_unconstrained(paddr, size, owner)
        .or(Err(PagingError::InternalFailure))
}

fn update_tracing_table(
    paddr:PhysicalAddress,
    size:u64,
    owner:MemoryOwner,
) -> Result<(),PagingError> {
    tracing::update::resize(paddr, size, owner)
}
