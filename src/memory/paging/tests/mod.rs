mod bitmap;
mod iterators;
mod management;

use crate::memory::{MemoryOwner, PageType, PhysicalAddress, SIZE_1GiB, SIZE_2MiB, SIZE_4KiB};
use crate::memory::map;
use crate::memory::paging::{self, Bitmap, PagingError, PageTable};
use crate::memory::paging::tracing;

pub(crate)
fn run() {
    bitmap::run_all_tests();
    iterators::run_indexers_tests();
    management::run_tables_tests();
    tracing::tests::run();
    management::run_pages_tests();
}


#[derive(Default, Clone, Copy)]
struct PageCount {
    pub(crate) n_pdpt : usize,
    pub(crate) n_pdt  : usize,
    pub(crate) n_pt   : usize,
    pub(crate) n_p_1g : usize,
    pub(crate) n_p_2m : usize,
    pub(crate) n_p_4k : usize,
}

impl PageCount {
    fn new(owner:MemoryOwner) -> Result<Self,PagingError> {
        let mut pages = Self::default();
        let pml4t = PageTable::pml4t(owner);
        for pml4t_entry in pml4t.iterate_allocations() {
            if !pml4t_entry.bitmap().present() {
                continue;
            }
            pages.n_pdpt += 1;
            let pdpt = PageTable::try_from(pml4t_entry)?;
            for pdpt_entry in pdpt.iterate_allocations() {
                let pdpt_entry_bits = pdpt_entry.bitmap();
                if !pdpt_entry_bits.present() {
                    continue;
                } else if pdpt_entry_bits.page_size() {
                    pages.n_p_1g += 1;
                    continue;
                }
                pages.n_pdt += 1;
                let pdt = PageTable::try_from(pdpt_entry)?;
                for pdt_entry in pdt.iterate_allocations() {
                    let pdt_entry_bits = pdt_entry.bitmap();
                    if !pdt_entry_bits.present() {
                        continue;
                    } else if pdt_entry_bits.page_size() {
                        pages.n_p_2m += 1;
                        continue;
                    }
                    pages.n_pt += 1;
                    let pt = PageTable::try_from(pdt_entry)?;
                    for pt_entry in pt.iterate_allocations() {
                        let pt_entry_bits = pt_entry.bitmap();
                        if !pt_entry_bits.present() {
                            continue;
                        }
                        pages.n_p_4k += 1;
                    }
                }
            }
        }
        Ok(pages)
    }
}

pub(in crate::memory)
fn check_pages(
    n_pdpt:usize,
    n_pdt:usize,
    n_pt:usize,
    n_p_1g:usize,
    n_p_2m:usize,
    n_p_4k:usize,
) -> bool {
    use crate::GetOrPanic;
    let pages = PageCount::new(MemoryOwner::Kernel).get_or_panic();
    return n_pdpt == pages.n_pdpt
        && n_pdt  == pages.n_pdt
        && n_pt   == pages.n_pt
        && n_p_1g == pages.n_p_1g
        && n_p_2m == pages.n_p_2m
        && n_p_4k == pages.n_p_4k;
}

pub(in crate::memory)
fn erase_pages() {
    use crate::{GetOrPanic, OkOrPanic};
    let owner = MemoryOwner::Kernel;
    let pml4t = PageTable::pml4t(owner);
    for pml4t_entry in pml4t.iterate_allocations() {
        if !pml4t_entry.bitmap().present() {
            continue;
        }
        let pdpt = PageTable::try_from(pml4t_entry.clone()).get_or_panic();
        let pdpt_base_address = pdpt.base_address;
        for pdpt_entry in pdpt.iterate_allocations() {
            let pdpt_entry_bits = pdpt_entry.bitmap();
            if !pdpt_entry_bits.present() {
                continue;
            } else if pdpt_entry_bits.page_size() {
                let page_paddr = PhysicalAddress::from(pdpt_entry_bits.address(PageType::OneGiB));
                map::release(page_paddr, PageType::OneGiB, owner).ok_or_panic();
                tracing::remove::remove_space(page_paddr, SIZE_1GiB, owner).ok_or_panic();
                pdpt_entry.set_bitmap(Bitmap::new());
                continue;
            }
            let pdt = PageTable::try_from(pdpt_entry.clone()).get_or_panic();
            let pdt_base_address = pdt.base_address;
            for pdt_entry in pdt.iterate_allocations() {
                let pdt_entry_bits = pdt_entry.bitmap();
                if !pdt_entry_bits.present() {
                    continue;
                } else if pdt_entry_bits.page_size() {
                    let page_paddr = PhysicalAddress::from(pdt_entry_bits.address(PageType::TwoMiB));
                    map::release(page_paddr, PageType::TwoMiB, owner).ok_or_panic();
                    tracing::remove::remove_space(page_paddr, SIZE_2MiB, owner).ok_or_panic();
                    pdt_entry.set_bitmap(Bitmap::new());
                    continue;
                }
                let pt = PageTable::try_from(pdt_entry.clone()).get_or_panic();
                let pt_base_address = pt.base_address;
                for pt_entry in pt.iterate_allocations() {
                    let pt_entry_bits = pt_entry.bitmap();
                    if !pt_entry_bits.present() {
                        continue;
                    }
                    let page_paddr = PhysicalAddress::from(pt_entry_bits.address(PageType::FourKiB));
                    map::release(page_paddr, PageType::FourKiB, owner).ok_or_panic();
                    tracing::remove::remove_space(page_paddr, SIZE_4KiB, owner).ok_or_panic();
                    pt_entry.set_bitmap(Bitmap::new());
                }
                map::release(pt_base_address, PageType::FourKiB, owner).ok_or_panic();
                pdt_entry.set_bitmap(Bitmap::new());
            }
            map::release(pdt_base_address, PageType::FourKiB, owner).ok_or_panic();
            pdpt_entry.set_bitmap(Bitmap::new());
        }
        map::release(pdpt_base_address, PageType::FourKiB, owner).ok_or_panic();
        pml4t_entry.set_bitmap(Bitmap::new());
    }
}
