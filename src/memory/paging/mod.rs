#![allow(non_upper_case_globals)]

pub(in crate::memory) mod bitmap;
pub(in crate::memory) mod iterators;
mod management;
pub(in crate::memory) mod page;
mod setup;
pub(in crate::memory) mod table;
pub(in crate::memory) mod tracing;
pub(in crate::memory) mod utilities;

#[cfg(feature="unit_tests")]
pub(crate) mod tests;

pub(in crate::memory) use bitmap::*;
pub(crate) use management::switch_page_map;
pub(in crate::memory) use management::clean;
pub(in crate::memory) use management::create;
pub(in crate::memory) use management::delete;
pub(in crate::memory) use management::drop;
pub(in crate::memory) use management::insert;
pub(in crate::memory) use management::query;
pub(in crate::memory) use management::remove;
pub(in crate::memory) use management::search;
pub(in crate::memory) use management::take;
pub(in crate::memory) use management::update;
pub(in crate::memory) use page::*;
pub(in crate::memory) use setup::init;
pub(crate) use setup::book_kernel_allocations_space;
pub(in crate::memory) use table::*;
pub(crate) use tracing::{TracingError, TracingPageError};
pub(crate) use tracing::init_kernel_tracing_pages;


use crate::{GetOrPanic, IntoOrPanic};
use crate::memory::MemoryOwner;
use crate::memory::{
    SIZE_8b, SIZE_4KiB, SIZE_2MiB, SIZE_1GiB,
    KERNEL_CODE_SIZE, KERNEL_STACK_SIZE, USER_STACK_SIZE,
};
use crate::memory::address;
use crate::panic::*;


//       UNUSED        PML4T     PDPT       PDT           OFFSET
//   0xFFFF/0x0000     0xFF8     0xFF0     0x000         0x000000
//  ???????????????? 111111111 111111110 000000000 000000000000000000000
//  #...#...#...#... F...F...F ...F...8. ..0...0.. .0...0...0...0...0...
const KERNEL_VADDR_BASE : u64 = 0xFFFFFFFF80000000;

//       UNUSED        PML4T     PDPT       PDT           OFFSET
//   0xFFFF/0x0000     0xFF8     0xFF0     0x000         0x000000
//  ???????????????? 111111111 111111110 111111111 111111111111111110000
//  #...#...#...#... F...F...F ...F...B. ..F...F.. .F...F...F...F...0...
const KERNEL_STACK_VADDR_BASE   : u64 = 0xFFFFFFFFBFFFFFF0;


// Start of memory
const ORIGIN_PADDR              : u64 = 0x0000000000000000;

// Address of the Page Map Level 4 Table
const KERNEL_PML4T_PADDR        : u64 = 0x0000000000100000;
// Address of the reserved Page Directory Pointer Table (used for the identity map)
const RESERVED_PDPT_PADDR       : u64 = 0x0000000000101000;
// Address of the kernel's Page Directory Pointer Table (used for the kernel code, stack and tracing pages)
const KERNEL_PDPT_PADDR         : u64 = 0x0000000000102000;
// Address of the kernel's Page Directory Table (used for the kernel code pages)
const KERNEL_CODE_PDT_PADDR     : u64 = 0x0000000000103000;
// Address of the kernel's Page Directory Table (used for the kernel stack pages)
const KERNEL_STACK_PDT_PADDR    : u64 = 0x0000000000104000;
// Address of the kernel's Page Directory Table (used for the kernel tracing pages)
const KERNEL_TRACING_PDT_PADDR  : u64 = 0x0000000000105000;

// Range reserved to store additional kernelspace page tables
const TABLES_KSPACE_BEG         : u64 = 0x0000000000106000;
const TABLES_KSPACE_END         : u64 = 0x0000000000110000;


pub(in crate::memory) const
N_KERNEL_CODE_PAGES     : u64 = KERNEL_CODE_SIZE / SIZE_2MiB;


pub(in crate::memory) const
N_KERNEL_STACK_PAGES    : u64 = KERNEL_STACK_SIZE / SIZE_2MiB;

pub(in crate::memory) const
N_USER_STACK_PAGES      : u64 = USER_STACK_SIZE / SIZE_2MiB;


/// The index of the first entry in a page table
pub(in crate::memory) const
FIRST_ENTRY_INDEX   : u64 = 0;

/// The offset of the entry in the middle of a page table
pub(in crate::memory) const
MIDDLE_ENTRY_INDEX  : u64 = 127;

/// The index of the last entry in a page table
pub(in crate::memory) const
LAST_ENTRY_INDEX    : u64 = 511;

/// The limit for the indexes of a page table
pub(in crate::memory) const
LIMIT_ENTRY_INDEX   : u64 = 512;

/// The offset of the first entry in a page table
pub(in crate::memory) const
FIRST_ENTRY_OFFSET  : u64 = 0x000;

/// The offset of the entry in the middle of a page table
pub(in crate::memory) const
MIDDLE_ENTRY_OFFSET : u64 = 0x800;

/// The offset of the last entry in a page table
pub(in crate::memory) const
LAST_ENTRY_OFFSET   : u64 = 0xFF8;

/// The limit for the offsets of a page table
pub(in crate::memory) const
LIMIT_ENTRY_OFFSET  : u64 = 0x1000;


pub(in crate::memory::paging) const
KERNEL_CODE_TABLES_PML4T_OFFSET         : u64 = 0xFF8;

pub(in crate::memory::paging) const
KERNEL_CODE_TABLES_PDPT_OFFSET          : u64 = 0xFF0;


pub(in crate::memory::paging) const
KERNEL_ALLOCATIONS_PML4T_FIRST_INDEX    : u64 = 1;

pub(in crate::memory::paging) const
KERNEL_ALLOCATIONS_PML4T_LIMIT_INDEX    : u64 = 510;

pub(in crate::memory::paging) const
KERNEL_ALLOCATIONS_PML4T_FIRST_OFFSET   : u64 = 0x008;

pub(in crate::memory::paging) const
KERNEL_ALLOCATIONS_PML4T_LIMIT_OFFSET   : u64 = 0xFF0;


pub(in crate::memory::paging) const
USER_ALLOCATIONS_PML4T_FIRST_INDEX      : u64 = 0;

pub(in crate::memory::paging) const
USER_ALLOCATIONS_PML4T_LIMIT_INDEX      : u64 = 511;

pub(in crate::memory::paging) const
USER_ALLOCATIONS_PML4T_FIRST_OFFSET     : u64 = 0x000;

pub(in crate::memory::paging) const
USER_ALLOCATIONS_PML4T_LIMIT_OFFSET     : u64 = 0xFF8;


pub(in crate::memory::paging) const
TRACING_TABLES_PML4T_OFFSET             : u64 = LAST_ENTRY_OFFSET;

pub(in crate::memory::paging) const
KERNEL_TRACING_TABLES_PDPT_FIRST_OFFSET : u64 = FIRST_ENTRY_OFFSET;

pub(in crate::memory::paging) const
KERNEL_TRACING_TABLES_PDPT_LIMIT_OFFSET : u64 = KERNEL_CODE_TABLES_PDPT_OFFSET;

pub(in crate::memory::paging) const
USER_TRACING_TABLES_PDPT_FIRST_OFFSET   : u64 = FIRST_ENTRY_OFFSET;

pub(in crate::memory::paging) const
USER_TRACING_TABLES_PDPT_LIMIT_OFFSET   : u64 = STACK_TABLES_PDPT_OFFSET;


pub(in crate::memory::paging) const
STACK_TABLES_PML4T_OFFSET               : u64 = LAST_ENTRY_OFFSET;

pub(in crate::memory::paging) const
STACK_TABLES_PDPT_OFFSET                : u64 = LAST_ENTRY_OFFSET;

pub(in crate::memory::paging) const
KERNEL_STACK_TABLES_PDT_FIRST_OFFSET    : u64 = MIDDLE_ENTRY_OFFSET - (KERNEL_STACK_SIZE / SIZE_2MiB / 2 * SIZE_8b);

pub(in crate::memory::paging) const
KERNEL_STACK_TABLES_PDT_LIMIT_OFFSET    : u64 = MIDDLE_ENTRY_OFFSET + (KERNEL_STACK_SIZE / SIZE_2MiB / 2 * SIZE_8b) + SIZE_8b;

pub(in crate::memory::paging) const
USER_STACK_TABLES_PDT_FIRST_OFFSET      : u64 = MIDDLE_ENTRY_OFFSET - (USER_STACK_SIZE / SIZE_2MiB / 2 * SIZE_8b);

pub(in crate::memory::paging) const
USER_STACK_TABLES_PDT_LIMIT_OFFSET      : u64 = MIDDLE_ENTRY_OFFSET + (USER_STACK_SIZE / SIZE_2MiB / 2 * SIZE_8b) + SIZE_8b;


pub(in crate::memory::paging) const
TRACING_TABLES_PML4T_INDEX              : u64 = LAST_ENTRY_INDEX;

pub(in crate::memory::paging) const
TRACING_TABLES_PDPT_INDEX               : u64 = FIRST_ENTRY_INDEX;


/// Describes an error occured while handling pages
pub(crate) enum PagingError {
    /// Something deadly wrong happened while processing
    InternalFailure,
    /// An entry index or offset is out of the bounds of a page or an iterator
    OutOfBounds,
    /// A page cannot be reached
    PageFault,
    /// The entry corresponding to the table does not have the present bit set
    TableNotPresent,
    /// The entry corresponding to the page does not have the present bit set
    PageNotPresent,
    /// The entry corresponding to the page does not have the page-size bit set
    PageNotHuge,
    /// Requested to insert a page which size is not compatible with the choosen table
    IncompatiblePage,
    /// Requested to insert a table which size is not compatible with the choosen table
    IncompatibleTable,
    /// Requested to insert an entry in a table which does not have any free slot
    NoSlotAvailable,
    /// Requested to allocate space for an entry, but there is no space available
    NoSpaceAvailable,
    /// Translating from logical to physical address failed during page allocation
    AddressTranslationFailed,
    /// Something failed while allocating memory for a new page or table
    AllocationFailure,
    /// Something failed while deallocating memory for a new page or table
    DeallocationFailure,
    /// Something wrong happened while cleaning up unused pages
    CleanupFailure,
    /// Something wrong happened while dealing with the tracing tables
    TracingError(tracing::TracingError),
    /// Error regarding pages addresses
    AddressError(address::AddressError),
    /// The request is not valid (to be only used when no other variant better applies)
    InvalidRequest,
}

impl From<tracing::TracingPageError> for PagingError {
    fn from(e:tracing::TracingPageError) -> Self {
        Self::from(TracingError::from(e))
    }
}

impl From<tracing::TracingError> for PagingError {
    fn from(e:tracing::TracingError) -> Self {
        Self::TracingError(e)
    }
}

impl From<address::AddressError> for PagingError {
    fn from(e:address::AddressError) -> Self {
        Self::AddressError(e)
    }
}

impl Panic for PagingError {
    fn panic(&self) -> ! {
        use PagingError::*;
        match self {
            InternalFailure          => panic("PagingError: InternalFailure"),
            OutOfBounds              => panic("PagingError: OutOfBounds"),
            PageFault                => panic("PagingError: PageFault"),
            TableNotPresent          => panic("PagingError: TableNotPresent"),
            PageNotPresent           => panic("PagingError: PageNotPresent"),
            PageNotHuge              => panic("PagingError: PageNotHuge"),
            IncompatiblePage         => panic("PagingError: IncompatiblePage"),
            IncompatibleTable        => panic("PagingError: IncompatibleTable"),
            NoSlotAvailable          => panic("PagingError: NoSlotAvailable"),
            NoSpaceAvailable         => panic("PagingError: NoSpaceAvailable"),
            AddressTranslationFailed => panic("PagingError: AddressTranslationFailed"),
            AllocationFailure        => panic("PagingError: AllocationFailure"),
            DeallocationFailure      => panic("PagingError: DeallocationFailure"),
            CleanupFailure           => panic("PagingError: CleanupFailure"),
            TracingError(_)          => panic("PagingError: TracingError"),
            AddressError(_)          => panic("PagingError: AddressError"),
            InvalidRequest           => panic("PagingError: InvalidRequest"),
        }
    }
}
