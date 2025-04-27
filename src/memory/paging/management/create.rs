// INITIAL PROCESS PAGES
//
//    RAM
//    ╔╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╗
//    ║#######################│#######│###################################################################║
//    ║#######################│ PML4T │###################################################################║
//    ╚╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧↓╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╝
//                                ┆
//        ┌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┘
//        ┆
//    ┏━━━∇━━━┯━━━━━━━┯━━━━━━━┯━━━━━━━┯━━━━━━━┓
//    ┃ 0x000 │  ...  │  ...  │  ...  │ 0xFF8 ┃
//    ┃ PDPT  │  ---  │  ---  │  ---  │ PDPT  ┃
//    ┗━━━⇓━━━┷━━━━━━━┷━━━━━━━┷━━━━━━━┷━━━⇓━━━┛
//        ┆                               ┆
//        ┆                               ┆
//    ┏━━━∇━━━┯━━━━━━━┯━━━━━━━┓       ┏━━━∇━━━┯━━━━━━━┯━━━━━━━┯━━━━━━━┓
//    ┃ 0x000 │  ...  │ 0xFF8 ┃       ┃ 0x000 │  ...  │  ...  │ 0xFF8 ┃
//    ┃  PDT  │  ---  │  ---  ┃       ┃  PDT  │  ---  │  ---  │  PDT  ┃
//    ┗━━━⇓━━━┷━━━━━━━┷━━━━━━━┛       ┗━━━↓━━━┷━━━━━━━┷━━━━━━━┷━━━↓━━━┛
//        ┆                               ┆                       ┆
//        ┆                               ┆                       └╌╌╌╌╌╌╌┐
//    ┏━━━∇━━━┯━━━━━━━┯━━━━━━━┓       ┏━━━∇━━━┯━━━━━━━┯━━━━━━━┓       ┏━━━∇━━━┯━━━━━━━┯━━━━━━━┯━━━━━━━┓
//    ┃ 0x000 │  ...  │ 0xFF8 ┃       ┃ 0x000 │  ...  │ 0xFF8 ┃       ┃ 0x000 │  ...  │  ...  │ 0xFF8 ┃
//    ┃  PT   │  ---  │  ---  ┃       ┃ 2 MiB │  ---  │  ---  ┃       ┃  ---  │  ---  │ 2 MiB │  ---  ┃
//    ┗━━━⇓━━━┷━━━━━━━┷━━━━━━━┛       ┗━━━━━━━┷━━━━━━━┷━━━━━━━┛       ┗━━━━━━━┷━━━━━━━┷━━━━━━━┷━━━━━━━┛
//        ┆
//        ┆
//    ┏━━━∇━━━┯━━━━━━━┯━━━━━━━┓
//    ┃ 0x000 │  ...  │ 0xFF8 ┃
//    ┃  ---  │  ---  │  ---  ┃
//    ┗━━━━━━━┷━━━━━━━┷━━━━━━━┛



use crate::{Log,LogError};
use crate::memory::*;
use crate::memory::address::*;
use crate::memory::map;
use crate::memory::memset;
use crate::memory::paging::*;
use crate::memory::paging::tracing::*;


const PAGE_TABLE_TYPE   : PageType = PageType::FourKiB;
const STACK_PAGE_TYPE   : PageType = PageType::TwoMiB;
const TRACE_PAGE_TYPE   : PageType = PageType::TwoMiB;

const PAGE_TABLE_SIZE   : u64 = PAGE_TABLE_TYPE.size();
const STACK_PAGE_SIZE   : u64 = STACK_PAGE_TYPE.size();
const TRACE_PAGE_SIZE   : u64 = TRACE_PAGE_TYPE.size();

const N_PAGE_TABLES     : u64 = 7;
const N_STACK_PAGES     : u64 = N_USER_STACK_PAGES;
const N_TRACE_PAGES     : u64 = 1;


/// Allocates and initializes a new paging structure in a contiguous
/// space in memory. This paging structure will be owned by a process.
///
/// This function will allocate a new PML4T, two new PDPT,
/// three new PDT, one new PT and five to nine 2MiB huge pages,
/// depending on the choosen stack size.
///
/// The PML4T will have the first PDPT as first entry, which will
/// have the first PDT as first entry, which will have the PT as
/// first entry, which won't pre-allocate any page.
/// These tables will be used by a process to store its private data.
///
/// The PML4T will also have the second PDPT as last entry, which
/// will have the second PDT as first entry, which will pre-allocate
/// one 2MiB huge page. These tables and page will be used by the
/// memory manager to track the memory allocations of the process.
///
/// The second PDPT will also have the third PDT al last entry, which
/// will have four to eight 2MiB pages as last entries.
/// These pages will be used as stack by the process.
///
/// The total allocated space will be 8 MiB + 32 KiB.
///
/// ## Returns
///
/// Returns [`Err`] if an error occured, otherwise returns [`Ok`] containing
/// the base address of the PML4T.
pub(in crate::memory)
fn create_paging_structure(
    owner:MemoryOwner,
) -> Result<PhysicalAddress, PagingError> {
    const tracing_owner : MemoryOwner = MemoryOwner::Kernel;

    let pml4t_paddr = map::find_available_range(PAGE_TABLE_TYPE, N_PAGE_TABLES)
        .ok_or(PagingError::NoSpaceAvailable)?;
    let tracepage_paddr = map::find_available_range(TRACE_PAGE_TYPE, N_TRACE_PAGES)
        .ok_or(PagingError::NoSpaceAvailable)?;
    let stack_paddr = map::find_available_range(STACK_PAGE_TYPE, N_STACK_PAGES)
        .ok_or(PagingError::NoSpaceAvailable)?;

    map::acquire_range(pml4t_paddr, PAGE_TABLE_TYPE, N_PAGE_TABLES, owner)
        .log_map_err(PagingError::AllocationFailure)?;
    map::acquire_range(tracepage_paddr, TRACE_PAGE_TYPE, N_TRACE_PAGES, tracing_owner)
        .or_else(|e| {
            e.log();
            map::release_range(tracepage_paddr, TRACE_PAGE_TYPE, N_TRACE_PAGES, tracing_owner)
                .log_map_err(PagingError::DeallocationFailure)
        })?;
    map::acquire_range(stack_paddr, STACK_PAGE_TYPE, N_STACK_PAGES, owner)
        .or_else(|e| {
            e.log();
            map::release_range(stack_paddr, STACK_PAGE_TYPE, N_STACK_PAGES, owner)
                .and_then(|()|
                    map::release_range(tracepage_paddr, TRACE_PAGE_TYPE, N_TRACE_PAGES, tracing_owner)
                ).log_map_err(PagingError::DeallocationFailure)
        })?;

    unsafe {
        memset(pml4t_paddr.get(), 0, PAGE_TABLE_SIZE*N_PAGE_TABLES);
        memset(stack_paddr.get(), 0, STACK_PAGE_SIZE*N_STACK_PAGES);
        memset(tracepage_paddr.get(), 0, TRACE_PAGE_SIZE*N_TRACE_PAGES);
    }

    create_process_pages(pml4t_paddr, owner);
    create_stack_pages(pml4t_paddr, stack_paddr, owner);
    create_tracing_pages(pml4t_paddr, tracepage_paddr, tracing_owner);

    Ok(pml4t_paddr)
}

fn create_process_pages(pml4t_paddr:PhysicalAddress, owner:MemoryOwner) {
    let process_flags = Bitmap::from(owner);
    let pml4t_entry_paddr = pml4t_paddr + USER_ALLOCATIONS_PML4T_FIRST_OFFSET;
    let pdpt_paddr = pml4t_paddr + SIZE_4KiB;
    PageTableEntry::new(pml4t_entry_paddr, PageTableType::PageMapLevel4Table, owner)
        .set_bitmap(Bitmap::from(pdpt_paddr) | process_flags);
    let pdpt_entry_paddr = pdpt_paddr + FIRST_ENTRY_OFFSET;
    let pdt_paddr = pdpt_paddr + SIZE_4KiB;
    PageTableEntry::new(pdpt_entry_paddr, PageTableType::PageDirectoryPointerTable, owner)
        .set_bitmap(Bitmap::from(pdt_paddr) | process_flags);
    let pdt_entry_paddr = pdt_paddr + FIRST_ENTRY_OFFSET;
    let pt_paddr = pdt_paddr + SIZE_4KiB;
    PageTableEntry::new(pdt_entry_paddr, PageTableType::PageDirectoryTable, owner)
        .set_bitmap(Bitmap::from(pt_paddr) | process_flags);
}

fn create_stack_pages(pml4t_paddr:PhysicalAddress, mut stack_paddr:PhysicalAddress, owner:MemoryOwner) {
    let stack_flags = Bitmap::from(owner);
    let pml4t_entry_paddr = pml4t_paddr + STACK_TABLES_PML4T_OFFSET;
    let pdpt_paddr = pml4t_paddr + (SIZE_4KiB * 4);
    PageTableEntry::new(pml4t_entry_paddr, PageTableType::PageMapLevel4Table, owner)
        .set_bitmap(Bitmap::from(pdpt_paddr) | stack_flags);
    let pdpt_entry_paddr = pdpt_paddr + STACK_TABLES_PDPT_OFFSET;
    let pdt_paddr = pdpt_paddr + (SIZE_4KiB * 2);
    PageTableEntry::new(pdpt_entry_paddr, PageTableType::PageDirectoryPointerTable, owner)
        .set_bitmap(Bitmap::from(pdt_paddr) | stack_flags);
    let mut pdt_entry_paddr = pdt_paddr + USER_STACK_TABLES_PDT_FIRST_OFFSET;
    for _ in 0..N_USER_STACK_PAGES {
        PageTableEntry::new(pdt_paddr, PageTableType::PageDirectoryTable, owner)
            .set_bitmap(Bitmap::from(stack_paddr) | stack_flags);
        pdt_entry_paddr += SIZE_8b;
        stack_paddr += SIZE_2MiB;
    }
}

fn create_tracing_pages(pml4t_paddr:PhysicalAddress, page_paddr:PhysicalAddress, owner:MemoryOwner) {
    let tracing_flags = Bitmap::from(owner);
    /* the PML4T entry is the same as the stack pages, so it has already been set-up */
    let pdpt_paddr = pml4t_paddr + (SIZE_4KiB * 4);
    let pdpt_entry_paddr = pdpt_paddr + USER_TRACING_TABLES_PDPT_FIRST_OFFSET;
    let pdt_paddr = pdpt_paddr + SIZE_4KiB;
    PageTableEntry::new(pdpt_entry_paddr, PageTableType::PageDirectoryPointerTable, owner)
        .set_bitmap(Bitmap::from(pdt_paddr) | tracing_flags);
    let pdt_entry_paddr = pdt_paddr + FIRST_ENTRY_OFFSET;
    PageTableEntry::new(pdt_entry_paddr, PageTableType::PageDirectoryTable, owner)
        .set_bitmap(Bitmap::from(page_paddr) | tracing_flags.with_bits(PS_BIT));
    TracingPage::init(page_paddr);
}
