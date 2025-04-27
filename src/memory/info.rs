use super::address::*;
use super::map;
use super::paging::PageType;


const MEMINFO_COUNT_PADDR       : u64 = 0x20000;
const MEMINFO_MAP_PADDR         : u64 = 0x20008;

const FREE_REGION               : u32 = 1;
const RESERVED_REGION           : u32 = 2;
const ACPI_RECLAIMABLE_REGION   : u32 = 3;
const ACPI_RESERVED_REGION      : u32 = 4;
const BAD_MEMORY_REGION         : u32 = 5;


/// Represents a macro-region of dynamic memory
#[repr(C,packed)]
struct MemoryRegion {
    /// The base address of the memory region
    base_paddr : u64,
    /// The size of the memory region
    size : u64,
    /// The memory type
    mem_type : u32,
}


/// Checks the memory regions to determine the available memory
///
/// ## Returns
///
/// Returns the highest possible physical address, namely the actual
/// memory size
pub(in crate::memory)
fn memory_size() -> u64 {
    let count = unsafe { *(MEMINFO_COUNT_PADDR as *const u32) } as usize;
    let mem_map = MEMINFO_MAP_PADDR as *mut MemoryRegion;

    #[cfg(feature="unit_tests")]
    let mut tot_mem_sz = 0_u64;
    #[cfg(feature="unit_tests")]
    let mut rsv_mem_sz = 0_u64;

    let mut last_paddr = 0_u64;
    for i in 0..count {
        let region_info = unsafe { &*mem_map.add(i) };
        if region_info.size == 0 {
            continue;
        }
        let end_paddr = region_info.base_paddr + region_info.size;
        if end_paddr > last_paddr {
            last_paddr = end_paddr;
        }

        #[cfg(feature="unit_tests")]
        {
            if region_info.mem_type != FREE_REGION {
                rsv_mem_sz += region_info.size;
            }
            tot_mem_sz += region_info.size;
        }
    }

    #[cfg(feature="unit_tests")]
    {
        crate::tty::print("TotalMemory: ");
        crate::tty::print_usize(tot_mem_sz as usize /1048576);
        crate::tty::print("MB - Free: ");
        crate::tty::print_usize((tot_mem_sz-rsv_mem_sz) as usize /1048576);
        crate::tty::print("MB - Reserved: ");
        crate::tty::print_usize(rsv_mem_sz as usize /1024);
        crate::tty::print("KB\n");
    }

    last_paddr
}


/// Parses the memory regions and updates the internal memory map
/// accordingly
///
/// Determines which memory regions are available to be used for allocating
/// and which are reserved and thus shall not be used
pub(in crate::memory)
fn parse_smap() {
    let count = unsafe { *(MEMINFO_COUNT_PADDR as *const u32) } as usize;
    let mem_map = MEMINFO_MAP_PADDR as *mut MemoryRegion;

    for i in 0..count {
        let region_info = unsafe { &*mem_map.add(i) };
        if region_info.size == 0 {
            continue;
        } else if region_info.mem_type == ACPI_RECLAIMABLE_REGION {
            // TODO: handle this case
        } else if region_info.mem_type == RESERVED_REGION {
            let beg_paddr = PhysicalAddress::from(region_info.base_paddr)
                .aligned_to_lower(PageType::FourKiB);
            let end_paddr = PhysicalAddress::from(region_info.base_paddr + region_info.size)
                .aligned_to_upper(PageType::FourKiB);
            map::set_reserved(beg_paddr, end_paddr);
        }

        #[cfg(feature="unit_tests")]
        {
            crate::tty::print_hex(region_info.base_paddr as usize);
            crate::tty::print("..");
            crate::tty::print_hex((region_info.base_paddr+region_info.size) as usize);
            crate::tty::print(" -> ");
            crate::tty::print_usize(region_info.mem_type as usize);
            crate::tty::print("\n");
        }
    }
}
