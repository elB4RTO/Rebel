use crate::test::*;
use crate::memory::paging::tests::{check_pages, erase_pages};
use crate::memory::paging::tracing::tests::check_tracing;

use crate::{GetOrPanic,OkOrPanic};
use crate::memory::paging::*;
use crate::memory::paging::iterators::AllocationsIterator;

/// # Tests
///
/// ### Pre-conditions
///
/// - The memory map is initialized
/// - The paging structure of the process is initialized
/// - The allocation tables of the process are clean
///
/// ### Post-conditions
///
/// - The allocation tables of the process are clean
pub(crate) fn run_tables_tests() {
    module!("memory::paging::management", "tables");
    // insert tables
    test::insert_pdpt();
    test::insert_pdt();
    test::insert_pt();
    // cleanup tables
    test::clean();
}

/// # Tests
///
/// ### Pre-conditions
///
/// - The memory map is initialized
/// - The paging structure of the process is initialized
/// - One tracing table is initialized for the process
/// - The allocation tables of the process are clean
///
/// ### Post-conditions
///
/// - The tracing table of the process is one and clean
/// - The allocation tables of the process are clean
pub(crate) fn run_pages_tests() {
    module!("memory::paging::management", "pages");
    // search pages slots
    erase_pages();
    test::fail__search_1gib_page_slot();
    test::search_1gib_page_slot();
    test::fail__search_2mib_page_slot();
    test::search_2mib_page_slot();
    test::fail__search_4kib_page_slot();
    test::search_4kib_page_slot();
    // insert pages
    test::insert_1gib_page();
    test::insert_2mib_page();
    test::insert_4kib_page();
    erase_pages();
    // force insert pages
    test::force_insert_1gib_page();
    erase_pages();
    test::force_insert_2mib_page();
    erase_pages();
    test::force_insert_4kib_page();
    erase_pages();
    // remove pages
    test::remove_1gib_page();
    erase_pages();
    test::remove_2mib_page();
    erase_pages();
    test::remove_4kib_page();
    erase_pages();
}

mod test {

use super::*;

const TABLE_PDPT    : PageTableType = PageTableType::PageDirectoryPointerTable;
const TABLE_PDT     : PageTableType = PageTableType::PageDirectoryTable;
const TABLE_PT      : PageTableType = PageTableType::PageTable;

const PAGE_1G       : PageType = PageType::OneGiB;
const PAGE_2M       : PageType = PageType::TwoMiB;
const PAGE_4K       : PageType = PageType::FourKiB;

const FLAGS         : Bitmap = Bitmap::default_kernel();
const FLAGS_HUGE    : Bitmap = Bitmap::default_kernel().with_bits(PS_BIT);

const OWNER         : MemoryOwner = MemoryOwner::Kernel;

/// # Test
///
/// ### Given
///
/// - A paging structure with unused pages
///
/// ### When
///
/// - Requesting to clean-up the unused pages from the structure
///
/// ### Then
///
/// - All the unused pages are deleted
pub(super) fn clean() {
    scenario!("Clean unused pages");
    // Given
    assume!(check_pages(0,0,0,0,0,0) == false, "pages pre-conditions");
    // When
    clean::cleanup_unused_pages(OWNER).ok_or_panic();
    // Then
    assume!(check_pages(0,0,0,0,0,0), "not all unused pages have been removed");
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A clean paging structure with no table or page
///
/// ### When
///
/// - Requesting to insert a PDPT
///
/// ### Then
///
/// - The PDPT is succesfully created
/// - No other table or page is created
pub(super) fn insert_pdpt() {
    scenario!("Insert a PDPT");
    // Given
    assume!(check_pages(0,0,0,0,0,0), "pages pre-conditions");
    assume!(check_tracing(0,0), "tracing pre-conditions");
    // When
    let table = insert::insert_table(TABLE_PDPT, FLAGS, OWNER).get_or_panic();
    // Then
    check!(table.is_some(), "table is none");
    assume!(check_pages(1,0,0,0,0,0), "pages post-conditions");
    assume!(check_tracing(0,0), "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A paging structure with one clean PDPT
///
/// ### When
///
/// - Requesting to insert a PDT
///
/// ### Then
///
/// - The PDT is succesfully created as an entry of the existing PDPT
/// - No other table or page is created
pub(super) fn insert_pdt() {
    scenario!("Insert a PDT");
    // Given
    assume!(check_pages(1,0,0,0,0,0), "pages pre-conditions");
    assume!(check_tracing(0,0), "tracing pre-conditions");
    // When
    let table = insert::insert_table(TABLE_PDT, FLAGS, OWNER).get_or_panic();
    // Then
    check!(table.is_some(), "table is none");
    assume!(check_pages(1,1,0,0,0,0), "pages post-conditions");
    assume!(check_tracing(0,0), "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A paging structure with one PDPT and one clean PDT
///
/// ### When
///
/// - Requesting to insert a PT
///
/// ### Then
///
/// - The PT is succesfully created as an entry of the existing PDT
/// - No other table or page is created
pub(super) fn insert_pt() {
    scenario!("Insert a PT");
    // Given
    assume!(check_pages(1,1,0,0,0,0), "pages pre-conditions");
    assume!(check_tracing(0,0), "tracing pre-conditions");
    // When
    let table = insert::insert_table(TABLE_PT, FLAGS, OWNER).get_or_panic();
    // Then
    check!(table.is_some(), "table is none");
    assume!(check_pages(1,1,1,0,0,0), "pages post-conditions");
    assume!(check_tracing(0,0), "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A clean paging structure with no table or page
///
/// ### When
///
/// - Requesting to find the slot for one 1 GiB huge page
///
/// ### Then
///
/// - No valid slots is found
/// - No table or page is created
pub(super) fn fail__search_1gib_page_slot() {
    fail_scenario!("Search a slot for one 1 GiB huge page");
    // Given
    assume!(check_pages(0,0,0,0,0,0), "pages pre-conditions");
    assume!(check_tracing(0,0), "tracing pre-conditions");
    // When
    let search_result = search::find_pages_slots(PAGE_1G, 1, OWNER);
    // Then
    check!(search_result.is_none(), "search result is Some");
    assume!(check_pages(0,0,0,0,0,0), "pages post-conditions");
    assume!(check_tracing(0,0), "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A paging structure with one PDPT
///
/// ### When
///
/// - Requesting to find the slot for one 1 GiB huge page
///
/// ### Then
///
/// - A valid slot is found
/// - No table or page is created
pub(super) fn search_1gib_page_slot() {
    scenario!("Search a slot for one 1 GiB huge page");
    // Given
    let _ = insert::insert_table(TABLE_PDPT, FLAGS, OWNER).get_or_panic();
    assume!(check_pages(1,0,0,0,0,0), "pages pre-conditions");
    assume!(check_tracing(0,0), "tracing pre-conditions");
    // When
    let search_result = search::find_pages_slots(PAGE_1G, 1, OWNER);
    // Then
    check!(search_result.is_some(), "search result is None");
    let mut table_iter = search_result.unwrap();
    match table_iter {
        AllocationsIterator::Pdpt(_) => (),
        _ => check!(false, "table type mismatch"),
    }
    let next_entry = table_iter.next();
    check!(next_entry.is_some(), "first iteration returns None");
    let (contiguous,first_table_entry) = next_entry.unwrap();
    check!(contiguous == true, "table entry continuity mismatch");
    check!(first_table_entry.bitmap().bits() == 0, "table entry bitmap mismatch");
    assume!(check_pages(1,0,0,0,0,0), "pages post-conditions");
    assume!(check_tracing(0,0), "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A paging structure with one PDPT
///
/// ### When
///
/// - Requesting to find the slot for one 2 MiB huge page
///
/// ### Then
///
/// - No valid slots is found
/// - No table or page is created
pub(super) fn fail__search_2mib_page_slot() {
    fail_scenario!("Search a slot for one 2 MiB huge page");
    // Given
    assume!(check_pages(1,0,0,0,0,0), "pages pre-conditions");
    assume!(check_tracing(0,0), "tracing pre-conditions");
    // When
    let search_result = search::find_pages_slots(PAGE_2M, 1, OWNER);
    // Then
    check!(search_result.is_none(), "search result is Some");
    assume!(check_pages(1,0,0,0,0,0), "pages post-conditions");
    assume!(check_tracing(0,0), "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A paging structure with one PDPT and one PDT
///
/// ### When
///
/// - Requesting to find the slot for one 2 MiB huge page
///
/// ### Then
///
/// - A valid slot is found
/// - No table or page is created
pub(super) fn search_2mib_page_slot() {
    scenario!("Search a slot for one 2 MiB huge page");
    // Given
    let _ = insert::insert_table(TABLE_PDT, FLAGS, OWNER).get_or_panic();
    assume!(check_pages(1,1,0,0,0,0), "pages pre-conditions");
    assume!(check_tracing(0,0), "tracing pre-conditions");
    // When
    let search_result = search::find_pages_slots(PAGE_2M, 1, OWNER);
    // Then
    check!(search_result.is_some(), "search result is None");
    let mut table_iter = search_result.unwrap();
    match table_iter {
        AllocationsIterator::Pdt(_) => (),
        _ => check!(false, "table type mismatch"),
    }
    let next_entry = table_iter.next();
    check!(next_entry.is_some(), "first iteration returns None");
    let (contiguous,first_table_entry) = next_entry.unwrap();
    check!(contiguous == true, "table entry continuity mismatch");
    check!(first_table_entry.bitmap().bits() == 0, "table entry bitmap mismatch");
    assume!(check_pages(1,1,0,0,0,0), "pages post-conditions");
    assume!(check_tracing(0,0), "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A paging structure with one PDPT and one PDT
///
/// ### When
///
/// - Requesting to find the slot for one 4 KiB page
///
/// ### Then
///
/// - No valid slots is found
/// - No table or page is created
pub(super) fn fail__search_4kib_page_slot() {
    fail_scenario!("Search a slot for one 4 KiB page");
    // Given
    assume!(check_pages(1,1,0,0,0,0), "pages pre-conditions");
    assume!(check_tracing(0,0), "tracing pre-conditions");
    // When
    let search_result = search::find_pages_slots(PAGE_4K, 1, OWNER);
    // Then
    check!(search_result.is_none(), "search result is Some");
    assume!(check_pages(1,1,0,0,0,0), "pages post-conditions");
    assume!(check_tracing(0,0), "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A paging structure with one PDPT, one PDT and one PT
///
/// ### When
///
/// - Requesting to find the slots for one 4 KiB huge page
///
/// ### Then
///
/// - A valid slot is found
/// - No table or page is created
pub(super) fn search_4kib_page_slot() {
    scenario!("Search a slot for one 4 KiB huge page");
    // Given
    let _ = insert::insert_table(TABLE_PT, FLAGS, OWNER).get_or_panic();
    assume!(check_pages(1,1,1,0,0,0), "pages pre-conditions");
    assume!(check_tracing(0,0), "tracing pre-conditions");
    // When
    let search_result = search::find_pages_slots(PAGE_4K, 1, OWNER);
    // Then
    check!(search_result.is_some(), "search result is None");
    let mut table_iter = search_result.unwrap();
    match table_iter {
        AllocationsIterator::Pt(_) => (),
        _ => check!(false, "table type mismatch"),
    }
    let next_entry = table_iter.next();
    check!(next_entry.is_some(), "first iteration returns None");
    let (contiguous,first_table_entry) = next_entry.unwrap();
    check!(contiguous == true, "table entry continuity mismatch");
    check!(first_table_entry.bitmap().bits() == 0, "table entry bitmap mismatch");
    assume!(check_pages(1,1,1,0,0,0), "pages post-conditions");
    assume!(check_tracing(0,0), "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A paging structure with one PDPT, one PDT and one PT
///
/// ### When
///
/// - Requesting to insert a 1 GiB huge page
///
/// ### Then
///
/// - The page is succesfully created as an entry of the existing PDPT
/// - No other table or page is created
pub(super) fn insert_1gib_page() {
    scenario!("Insert a 1 GiB huge page");
    // Given
    assume!(check_pages(1,1,1,0,0,0), "pages pre-conditions");
    assume!(check_tracing(0,0), "tracing pre-conditions");
    // When
    let page_addr = insert::insert_page(PAGE_1G, FLAGS_HUGE, OWNER).get_or_panic();
    // Then
    check!(page_addr.is_some(), "page is none");
    assume!(check_pages(1,1,1,1,0,0), "pages post-conditions");
    assume!(check_tracing(1,0), "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A paging structure with one PDPT, one PDT and one PT
///
/// ### When
///
/// - Requesting to insert a 2 MiB huge page
///
/// ### Then
///
/// - The page is succesfully created as an entry of the existing PDT
/// - No other table or page is created
pub(super) fn insert_2mib_page() {
    scenario!("Insert a 2 MiB huge page");
    // Given
    assume!(check_pages(1,1,1,1,0,0), "pages pre-conditions");
    assume!(check_tracing(1,0), "tracing pre-conditions");
    // When
    let page_addr = insert::insert_page(PAGE_2M, FLAGS_HUGE, OWNER).get_or_panic();
    // Then
    check!(page_addr.is_some(), "page is none");
    assume!(check_pages(1,1,1,1,1,0), "pages post-conditions");
    assume!(check_tracing(2,0), "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A paging structure with one PDPT, one PDT and one PT
///
/// ### When
///
/// - Requesting to insert a 4 KiB page
///
/// ### Then
///
/// - The page is succesfully created as an entry of the existing PT
/// - No other table or page is created
pub(super) fn insert_4kib_page() {
    scenario!("Insert a 4 KiB page");
    // Given
    assume!(check_pages(1,1,1,1,1,0), "pages pre-conditions");
    assume!(check_tracing(2,0), "tracing pre-conditions");
    // When
    let page_addr = insert::insert_page(PAGE_4K, FLAGS, OWNER).get_or_panic();
    // Then
    check!(page_addr.is_some(), "page is none");
    assume!(check_pages(1,1,1,1,1,1), "pages post-conditions");
    assume!(check_tracing(3,0), "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A clean paging structure with no table or page
///
/// ### When
///
/// - Requesting to force insert a 1 GiB huge page
///
/// ### Then
///
/// - The page is succesfully created
/// - A new PDPT is created in order to store the page
/// - No other table or page is created
pub(super) fn force_insert_1gib_page() {
    scenario!("Force insert a 1 GiB huge page");
    // Given
    assume!(check_pages(0,0,0,0,0,0), "pages pre-conditions");
    assume!(check_tracing(0,0), "tracing pre-conditions");
    // When
    let page_addr = insert::force_insert_pages(PAGE_1G, 1, FLAGS_HUGE, OWNER).get_or_panic();
    // Then
    assume!(check_pages(1,0,0,1,0,0), "pages post-conditions");
    assume!(check_tracing(1,0), "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A clean paging structure with no table or page
///
/// ### When
///
/// - Requesting to force insert a 2 MiB huge page
///
/// ### Then
///
/// - The page is succesfully created
/// - A new PDPT and a new PDT are created in order to store the page
/// - No other table or page is created
pub(super) fn force_insert_2mib_page() {
    scenario!("Force insert a 2 MiB huge page");
    // Given
    assume!(check_pages(0,0,0,0,0,0), "pages pre-conditions");
    assume!(check_tracing(0,0), "tracing pre-conditions");
    // When
    let page_addr = insert::force_insert_pages(PAGE_2M, 1, FLAGS_HUGE, OWNER).get_or_panic();
    // Then
    assume!(check_pages(1,1,0,0,1,0), "pages post-conditions");
    assume!(check_tracing(1,0), "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A clean paging structure with no table or page
///
/// ### When
///
/// - Requesting to force insert a 4 KiB page
///
/// ### Then
///
/// - The page is succesfully created
/// - A new PDPT, a new PDT and a new PT are created in order to store the page
/// - No other table or page is created
pub(super) fn force_insert_4kib_page() {
    scenario!("Force insert a 4 KiB page");
    // Given
    assume!(check_pages(0,0,0,0,0,0), "pages pre-conditions");
    assume!(check_tracing(0,0), "tracing pre-conditions");
    // When
    let page_addr = insert::force_insert_pages(PAGE_4K, 1, FLAGS, OWNER).get_or_panic();
    // Then
    assume!(check_pages(1,1,1,0,0,1), "pages post-conditions");
    assume!(check_tracing(1,0), "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A paging structure with one PDPT and one 1 GiB huge page
///
/// ### When
///
/// - Requesting to remove the 1 GiB huge page
///
/// ### Then
///
/// - The page is succesfully removed
/// - No table is removed
pub(super) fn remove_1gib_page() {
    scenario!("Remove a 1 GiB huge page");
    // Given
    let page_addr = insert::force_insert_pages(PAGE_1G, 1, FLAGS_HUGE, OWNER).get_or_panic();
    assume!(check_pages(1,0,0,1,0,0), "pages pre-conditions");
    assume!(check_tracing(1,0), "tracing pre-conditions");
    // When
    remove::remove_page(page_addr.logical, PAGE_1G, OWNER).ok_or_panic();
    // Then
    assume!(check_pages(1,0,0,0,0,0), "pages post-conditions");
    assume!(check_tracing(0,0), "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A paging structure with one PDPT, one PDT and one 2 MiB huge page
///
/// ### When
///
/// - Requesting to remove the 2 MiB huge page
///
/// ### Then
///
/// - The page is succesfully removed
/// - No table is removed
pub(super) fn remove_2mib_page() {
    scenario!("Remove a 2 MiB huge page");
    // Given
    let page_addr = insert::force_insert_pages(PAGE_2M, 1, FLAGS_HUGE, OWNER).get_or_panic();
    assume!(check_pages(1,1,0,0,1,0), "pages pre-conditions");
    assume!(check_tracing(1,0), "tracing pre-conditions");
    // When
    remove::remove_page(page_addr.logical, PAGE_2M, OWNER).ok_or_panic();
    // Then
    assume!(check_pages(1,1,0,0,0,0), "pages post-conditions");
    assume!(check_tracing(0,0), "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A paging structure with one PDPT, one PDT, one PT and one 4 KiB page
///
/// ### When
///
/// - Requesting to remove the 4 KiB page
///
/// ### Then
///
/// - The page is succesfully removed
/// - No table is removed
pub(super) fn remove_4kib_page() {
    scenario!("Remove a 4 KiB page");
    // Given
    let page_addr = insert::force_insert_pages(PAGE_4K, 1, FLAGS, OWNER).get_or_panic();
    assume!(check_pages(1,1,1,0,0,1), "pages pre-conditions");
    assume!(check_tracing(1,0), "tracing pre-conditions");
    // When
    remove::remove_page(page_addr.logical, PAGE_4K, OWNER).ok_or_panic();
    // Then
    assume!(check_pages(1,1,1,0,0,0), "pages post-conditions");
    assume!(check_tracing(0,0), "tracing post-conditions");
    test_passed!();
    wait!();
}

}
