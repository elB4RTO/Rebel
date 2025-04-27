// INITIAL KERNEL PAGES
//
//    RAM
//    ╔╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╤╗
//    ║#######│#######│####################################################################################║
//    ║#######│ PML4T │####################################################################################║
//    ╚∆╧╧╧╧╧╧╧╧╧╧↓╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╧╝
// ┌╌╌╌┘          ┆
// ┆              ┆
// ┆      ┌╌╌╌╌╌╌╌┘
// ┆  ┏━━━∇━━━┯━━━━━━━┯━━━━━━━┯━━━━━━━┯━━━━━━━┯━━━━━━━┓
// ┆  ┃ 0x000 │ 0x008 │  ...  │  ...  │ 0xFF0 │ 0xFF8 ┃
// ┆  ┃ PDPT  │  ---  │  ---  │  ---  │  ---  │ PDPT  ┃
// ┆  ┗━━━⇓━━━┷━━━━━━━┷━━━━━━━┷━━━━━━━┷━━━━━━━┷━━━⇓━━━┛
// ┆      ┆                                       ┆
// ┆      ┆                                       ┆
// ┆  ┏━━━∇━━━┯━━━━━━━┯━━━━━━━┯━━━━━━━┓       ┏━━━∇━━━┯━━━━━━━┯━━━━━━━┯━━━━━━━┓
// ┆  ┃ 0x000 │  ...  │  ...  │ 0xFF8 ┃       ┃ 0x000 │  ...  │ 0xFF0 │ 0xFF8 ┃
// ┆  ┃ 1 GiB │  ---  │  ---  │  ---  ┃       ┃  PDT  │  ---  │  PDT  │  PDT  ┃
// ┆  ┗━━━⇓━━━┷━━━━━━━┷━━━━━━━┷━━━━━━━┛       ┗━━━↓━━━┷━━━━━━━┷━━━⇓━━━┷━━━↓━━━┛
// └╌╌╌╌╌╌┘       ┌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┘               ┆       ┆
//                ┆                               ┌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┘       └╌╌╌╌╌╌╌┐
//            ┏━━━∇━━━┯━━━━━━━┯━━━━━━━┓       ┏━━━∇━━━┯━━━━━━━┯━━━━━━━┓       ┏━━━∇━━━┯━━━━━━━┯━━━━━━━┓
//            ┃ 0x000 │  ...  │ 0xFF8 ┃       ┃ 0x000 │  ...  │ 0xFF8 ┃       ┃ 0x000 │  ...  │ 0xFF8 ┃
//            ┃ 2 MiB │  ---  │  ---  ┃       ┃ 2 MiB │  ---  │  ---  ┃       ┃  ---  │ 2 MiB │  ---  ┃
//            ┗━━━━━━━┷━━━━━━━┷━━━━━━━┛       ┗━━━━━━━┷━━━━━━━┷━━━━━━━┛       ┗━━━━━━━┷━━━━━━━┷━━━━━━━┛



use crate::memory::KERNEL_PADDR_BASE;
use crate::memory::paging::*;


/// Initializes the bit masks used to extract the address bits
/// out of a bitmap
fn init_address_masks() {
    let (psz, _) = crate::cpu::features::pae_address_width();
    let mut addr_mask = 0_u64;
    for b in 0..psz {
        addr_mask |= 1 << b;
    }
    unsafe {
        BITMASK_ADDRESS_1G |= addr_mask & 0b1111111111111111111111111111111111000000000000000000000000000000;
        BITMASK_ADDRESS_2M |= addr_mask & 0b1111111111111111111111111111111111111111111000000000000000000000;
        BITMASK_ADDRESS_4K |= addr_mask & 0b1111111111111111111111111111111111111111111111111111000000000000;
    }
}

/// Queries the base address of the `Page Map Level 4 Table`
fn get_pml4t_base_address() {
    unsafe {
        let addr = crate::cpu::registers::get_cr3();
        if addr == ORIGIN_PADDR {
            crate::panic("Invalid PML4T address");
        }
    }
}

/// Identity-maps the first GiB of memory
///
/// The first entry of the PML4T will point to a PDPT that is located
/// exactly 0x1000 after it.
/// The first entry of the PDPT will be a huge 1 GiB page pointing to
/// the start of memory (0x0000000000000000).
///
/// Requires a total of 8 KiB of contiguous memory to store the
/// directory tables.
///
/// ## Map (logical -> physical)
///
/// 0x0000000000000000~0x000000003FFFFFFF -> 0x0000000000000000~0x000000003FFFFFFF
fn setup_identity_map() {
    unsafe {
        let mut page_dir_ptr = (KERNEL_PML4T_PADDR + FIRST_ENTRY_OFFSET) as *mut u64;
        let mut next = RESERVED_PDPT_PADDR;

        *page_dir_ptr = next | RW_BIT | P_BIT;

        page_dir_ptr = next as *mut u64;

        *page_dir_ptr = ORIGIN_PADDR | PS_BIT | RW_BIT | P_BIT;
    }
}

/// Sets-up the main kernel's pages
///
/// The last entry of the PML4T will point to the kernel's PDPT, that is located
/// exactly 0x2000 after the PML4T.
/// The second-last entry of the PDPT will point to the kernel's PDT, that is located
/// exactly 0x1000 after it.
/// The first 32 entries of the PDT will be a series of huge 2 MiB pages pointing to
/// the location of the kernel in memory.
///
/// Requires a total of 16 KiB of contiguous memory to store the
/// directory tables (4 KiB of which are not directly used, since they
/// hold the PDPT of the identity map), plus 64 MiB of available and
/// contiguous space for the pages.
///
/// This setup makes the base logical address of the kernel be 0xFFFFFFFF80000000.
///
/// ## Map (logical -> physical)
///
/// 0xFFFFFFFF80000000-0xFFFFFFFF801FFFFF -> 0x0000000001200000-0x00000000013FFFFF
/// 0xFFFFFFFF80200000-0xFFFFFFFF803FFFFF -> 0x0000000001400000-0x00000000015FFFFF
/// ...
/// 0xFFFFFFFF83C00000-0xFFFFFFF83DFFFFFF -> 0x0000000004800000-0x00000000049FFFFF
/// 0xFFFFFFFF83E00000-0xFFFFFFF83FFFFFFF -> 0x0000000005000000-0x00000000051FFFFF
fn setup_kernel_code_pages() {
    unsafe {
        let mut page_dir_ptr = (KERNEL_PML4T_PADDR + KERNEL_CODE_TABLES_PML4T_OFFSET) as *mut u64;
        let mut next = KERNEL_PDPT_PADDR;

        // set the PML4T entry to point to the PDPT
        *page_dir_ptr = next | RW_BIT | P_BIT;

        page_dir_ptr = (next + KERNEL_CODE_TABLES_PDPT_OFFSET) as *mut u64;
        next = KERNEL_CODE_PDT_PADDR;

        // set the PDPT entry to point to the PDT
        *page_dir_ptr = next | RW_BIT | P_BIT;

        let mut page_dir = next;
        next = KERNEL_PADDR_BASE;

        for _ in 0..N_KERNEL_CODE_PAGES {
            page_dir_ptr = page_dir as *mut u64;

            // set the PDT entry to point to the page
            *page_dir_ptr = next | PS_BIT | RW_BIT | P_BIT;

            page_dir += SIZE_8b;
            next += SIZE_2MiB;
        }
    }
}

/// Sets-up the pages used for the kernel's stack
///
/// The last 8 entries of the kernel's PDT will point to 8 huge 2 MiB pages,
/// starting 16 MiB below the main page and up to the start of the main page itself.
///
/// This function assumes that the PDPT and the PDT has already been set-up.
///
/// ## Map (logical -> physical)
///
/// 0xFFFFFFFFBFFFFFFF~0xFFFFFFFFBFE00000 -> 0x00000000011FFFFF-0x0000000001000000
/// 0xFFFFFFFFBDFFFFFF~0xFFFFFFFFBFC00000 -> 0x00000000009FFFFF-0x0000000000800000
/// ...
/// 0xFFFFFFFFBF3FEFFF-0xFFFFFFFFBF200000 -> 0x00000000005FFFFF-0x0000000000400000
/// 0xFFFFFFFFBF1FFFFF-0xFFFFFFFFBF000000 -> 0x00000000003FFFFF-0x0000000000200000
fn setup_kernel_stack_pages() {
    unsafe {
        let mut page_dir = KERNEL_STACK_PDT_PADDR + KERNEL_STACK_TABLES_PDT_LIMIT_OFFSET;
        let mut next = KERNEL_PADDR_BASE;

        for _ in 0..N_KERNEL_STACK_PAGES {
            page_dir -= SIZE_8b;
            let page_dir_ptr = page_dir as *mut u64;
            next -= SIZE_2MiB;

            *page_dir_ptr = next | PS_BIT | RW_BIT | P_BIT;
        }
    }
}

/// Initializes the paging tables and all the related stuff
///
/// ## Panics
///
/// This function will lead to a kernel panic in case the
/// PML4T address is invalid
pub(in crate::memory) fn init() {
    crate::tty::print("[MEMORY]> Initializing paging structure\n");

    init_address_masks();

    // NOTE:
    //  Currently does basically nothing, apart from fetching CR3 and discard the result.
    //  Need to clarify whether to keep using a selfmade bootloader and constant addresses
    //  or to find a premade one and determine addresses at runtime
    get_pml4t_base_address();

    setup_identity_map();

    setup_kernel_code_pages();

    setup_kernel_stack_pages();
}


pub(crate)
fn book_kernel_allocations_space() {
    crate::tty::print("[MEMORY]> Initializing allocation pages\n");

    // create the kernel's pages used to allocate objects in memory
    let flags = Bitmap::default_kernel().with_bits(PS_BIT);
    insert::force_insert_pages(PageType::TwoMiB, 1, flags, MemoryOwner::Kernel).get_or_panic();
}
