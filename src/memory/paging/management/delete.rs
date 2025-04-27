use crate::LogError;
use crate::memory::address::*;
use crate::memory::paging::*;
use crate::memory::map;


/// De-allocates and de-initializes the whole paging structure, including
/// the tracing pages
///
/// ## Returns
///
/// Returns an empty [`Ok`] if no error occured while processing, otherwise
/// returns an [`Err`] containing the error.
pub(in crate::memory)
fn delete_paging_structure(
    owner:MemoryOwner,
) -> Result<(), PagingError> {
    let mut failed = false;
    let pml4t = PageTable::pml4t(owner);
    let pml4t_address = pml4t.base_address;
    for pml4t_entry in pml4t.iterate_unbounded() {
        if !pml4t_entry.bitmap().present() {
            continue;
        }
        let pdpt = PageTable::try_from(pml4t_entry.clone())?;
        let pdpt_address = pdpt.base_address;
        for pdpt_entry in pdpt.iterate_unbounded() {
            let pdpt_entry_bits = pdpt_entry.bitmap();
            if !pdpt_entry_bits.present() {
                continue;
            }
            if pdpt_entry_bits.page_size() {
                let page_paddr = PhysicalAddress::from(pdpt_entry_bits.address(PageType::OneGiB));
                if let Err(_) = map::release(page_paddr, PageType::OneGiB, owner).log_err() {
                    failed |= true;
                }
                continue;
            }
            let pdt = PageTable::try_from(pdpt_entry.clone())?;
            let pdt_address = pdt.base_address;
            for pdt_entry in pdt.iterate_unbounded() {
                let pdt_entry_bits = pdt_entry.bitmap();
                if !pdt_entry_bits.present() {
                    continue;
                }
                if pdt_entry_bits.page_size() {
                    let page_paddr = PhysicalAddress::from(pdt_entry_bits.address(PageType::TwoMiB));
                    if let Err(_) = map::release(page_paddr, PageType::TwoMiB, owner).log_err() {
                        failed |= true;
                    }
                    continue;
                }
                let pt = PageTable::try_from(pdt_entry.clone())?;
                let pt_address = pt.base_address;
                for pt_entry in pt.iterate_unbounded() {
                    let pt_entry_bits = pt_entry.bitmap();
                    if !pt_entry_bits.present() {
                        continue;
                    }
                    let page_paddr = PhysicalAddress::from(pt_entry_bits.address(PageType::FourKiB));
                    if let Err(_) = map::release(page_paddr, PageType::FourKiB, owner).log_err() {
                        failed |= true;
                    }
                    pt_entry.set_bitmap(Bitmap::new());
                }
                if let Err(_) = map::release(pt_address, PageType::FourKiB, owner).log_err() {
                    failed |= true;
                }
                pdt_entry.set_bitmap(Bitmap::new());
            }
            if let Err(_) = map::release(pdt_address, PageType::FourKiB, owner).log_err() {
                failed |= true;
            }
            pdpt_entry.set_bitmap(Bitmap::new());
        }
        if let Err(_) = map::release(pdpt_address, PageType::FourKiB, owner).log_err() {
            failed |= true;
        }
        pml4t_entry.set_bitmap(Bitmap::new());
    }
    if let Err(_) = map::release(pml4t_address, PageType::FourKiB, owner).log_err() {
        failed |= true;
    }
    match failed {
        true  => Err(PagingError::DeallocationFailure),
        false => Ok(()),
    }
}
