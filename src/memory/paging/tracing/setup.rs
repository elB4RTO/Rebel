use crate::memory::{Init, MemoryOwner};
use crate::memory::address::*;
use crate::memory::map;
use crate::memory::memset;
use crate::memory::paging::*;
use crate::memory::paging::tracing::TracingPage;


/// Initializes the kernel's tracing pages
pub(crate)
fn init_kernel_tracing_pages() {
    crate::tty::print("[MEMORY]> Initializing tracing pages\n");

    let table_paddr = map::find_available(PageType::FourKiB)
        .ok_or(PagingError::NoSpaceAvailable)
        .get_or_panic();
    map::acquire(table_paddr, PageType::FourKiB, MemoryOwner::Kernel)
        .get_or_panic();

    let page_paddr = map::find_available(PageType::TwoMiB)
        .ok_or(PagingError::NoSpaceAvailable)
        .get_or_panic();
    map::acquire(page_paddr, PageType::TwoMiB, MemoryOwner::Kernel)
        .get_or_panic();

    unsafe {
        memset(table_paddr.get(), 0, SIZE_4KiB);
        memset(page_paddr.get(), 0, SIZE_2MiB);
    }

    let flags = Bitmap::default_kernel();
    let kernel_pdpt_entry_paddr = PhysicalAddress::from(KERNEL_PDPT_PADDR);
    PageTableEntry::new(kernel_pdpt_entry_paddr, PageTableType::PageDirectoryPointerTable, MemoryOwner::Kernel)
        .set_bitmap(Bitmap::from(table_paddr) | flags);
    PageTableEntry::new(table_paddr, PageTableType::PageDirectoryTable, MemoryOwner::Kernel)
        .set_bitmap(Bitmap::from(page_paddr) | flags.with_bits(PS_BIT));
    TracingPage::init(page_paddr);
}
