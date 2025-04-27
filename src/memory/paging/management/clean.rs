use crate::LogError;
use crate::memory::address::*;
use crate::memory::paging::*;
use crate::memory::paging::iterators::PageTableIterator;
use crate::memory::map;


/// Unsets the unused pages and table entries
///
/// A page is considered unused if there's nothing allocated in it.
/// A table entry is considered unused if there's nothing allocated
/// in all of its entries
///
/// ## Returns
///
/// Returns an [`Err`] if something fails during the cleanup (despite that,
/// the cleanup process won't stop, it will just skip cleaning pages and
/// entries that resulted in a failure of some kind)
pub(in crate::memory)
fn cleanup_unused_pages(
    owner:MemoryOwner,
) -> Result<(), PagingError> {
    let ref mut failed = false;
    let pml4t = PageTable::pml4t(owner);
    for pml4t_entry in pml4t.iterate_allocations().reversed() {
        if !pml4t_entry.bitmap().present() {
            continue;
        }
        let pdpt = PageTable::try_from(pml4t_entry.clone())?;
        let pdpt_address = pdpt.base_address;
        if cleanup_pdpt(pdpt.iterate_allocations().reversed(), owner, failed)? {
            pml4t_entry.set_bitmap(Bitmap::new());
            if let Err(_) = map::release(pdpt_address, PageType::FourKiB, owner).log_err() {
                *failed |= true;
            }
        }
    }
    match failed {
        true  => Err(PagingError::CleanupFailure),
        false => Ok(()),
    }
}

/// Cleans-up all the entries of the PDPT table iterated by `pdpt_iter`
fn cleanup_pdpt(pdpt_iter:PageTableIterator, owner: MemoryOwner, failed:&mut bool) -> Result<bool, PagingError> {
    let mut clean_pml4t_entry = true;
    for pdpt_entry in pdpt_iter {
        let pdpt_entry_bits = pdpt_entry.bitmap();
        if !pdpt_entry_bits.present() {
            continue;
        }
        if pdpt_entry_bits.page_size() {
            let page_paddr = PhysicalAddress::from(pdpt_entry_bits.address(PageType::OneGiB));
            match map::has_space(page_paddr, PageType::OneGiB, SIZE_1GiB, owner).log_err() {
                Ok(true) => match map::release(page_paddr, PageType::OneGiB, owner).log_err() {
                    Ok(()) => match tracing::remove::remove_space(page_paddr, SIZE_1GiB, owner).log_err() {
                        Ok(()) => pdpt_entry.set_bitmap(Bitmap::new()),
                        Err(_) => *failed |= true,
                    },
                    Err(_) => *failed |= true,
                },
                Ok(false) => clean_pml4t_entry &= false,
                Err(_) => *failed |= true,
            }
            continue;
        }
        let pdt = PageTable::try_from(pdpt_entry.clone())?;
        let pdt_address = pdt.base_address;
        match cleanup_pdt(pdt.iterate_allocations().reversed(), owner, failed)? {
            true => {
                pdpt_entry.set_bitmap(Bitmap::new());
                if let Err(_) = map::release(pdt_address, PageType::FourKiB, owner).log_err() {
                    *failed |= true;
                }
            },
            false => clean_pml4t_entry &= false,
        }
    }
    Ok(clean_pml4t_entry)
}

/// Cleans-up all the entries of the PDT table iterated by `pdt_iter`
fn cleanup_pdt(pdt_iter:PageTableIterator, owner: MemoryOwner, failed:&mut bool) -> Result<bool, PagingError> {
    let mut clean_pdpt_entry = true;
    for pdt_entry in pdt_iter {
        let pdt_entry_bits = pdt_entry.bitmap();
        if !pdt_entry_bits.present() {
            continue;
        }
        if pdt_entry_bits.page_size() {
            let page_paddr = PhysicalAddress::from(pdt_entry_bits.address(PageType::TwoMiB));
            match map::has_space(page_paddr, PageType::TwoMiB, SIZE_2MiB, owner).log_err() {
                Ok(true) => match map::release(page_paddr, PageType::TwoMiB, owner).log_err() {
                    Ok(()) => match tracing::remove::remove_space(page_paddr, SIZE_2MiB, owner).log_err() {
                        Ok(()) => pdt_entry.set_bitmap(Bitmap::new()),
                        Err(_) => *failed |= true,
                    },
                    Err(_) => *failed |= true,
                },
                Ok(false) => clean_pdpt_entry &= false,
                Err(_) => *failed |= true,
            }
            continue;
        }
        let pt = PageTable::try_from(pdt_entry.clone())?;
        let pt_address = pt.base_address;
        match cleanup_pt(pt.iterate_allocations().reversed(), owner, failed) {
            true => {
                pdt_entry.set_bitmap(Bitmap::new());
                if let Err(_) = map::release(pt_address, PageType::FourKiB, owner).log_err() {
                    *failed |= true;
                }
            },
            false => clean_pdpt_entry &= false,
        }
    }
    Ok(clean_pdpt_entry)
}

/// Cleans-up all the entries of the PT table iterated by `pt_iter`
fn cleanup_pt(pt_iter:PageTableIterator, owner: MemoryOwner, failed:&mut bool) -> bool {
    let mut clean_pdt_entry = true;
    for pt_entry in pt_iter {
        let pt_entry_bits = pt_entry.bitmap();
        if !pt_entry_bits.present() {
            continue;
        }
        let page_paddr = PhysicalAddress::from(pt_entry_bits.address(PageType::FourKiB));
        match map::has_space(page_paddr, PageType::FourKiB, SIZE_4KiB, owner).log_err() {
            Ok(true) => match map::release(page_paddr, PageType::FourKiB, owner).log_err() {
                Ok(()) => match tracing::remove::remove_space(page_paddr, SIZE_4KiB, owner).log_err() {
                    Ok(()) => pt_entry.set_bitmap(Bitmap::new()),
                    Err(_) => *failed |= true,
                },
                Err(_) => *failed |= true,
            },
            Ok(false) => clean_pdt_entry &= false,
            Err(_) => *failed |= true,
        }
    }
    clean_pdt_entry
}
