use crate::test::*;
use crate::memory::paging::tracing::tests::*;

use crate::GetOrPanic;
use crate::memory::{SIZE_2MiB, MemoryOwner, LogicalAddress, PhysicalAddress};
use crate::memory::paging::tracing::*;
use crate::memory::paging::tracing::management::*;

/// # Tests
///
/// ### Pre-conditions
///
/// - The memory map is initialized
/// - The paging structure of the process is initialized
/// - One tracing table is initialized for the process
///
/// ### Post-conditions
///
/// - The tracing table of the process is one and clean
pub(crate)
fn run_all_tests() {
    module!("memory::paging::tracing", "management");

    // create pages
    test::create_new_page();
    // delete pages
    test::delete_page();
    // cleanup pages
    test::cleanup_unused_pages_two_empty();
    test::cleanup_unused_pages_two_below();
    test::cleanup_unused_pages_two_above();
    test::cleanup_unused_pages_three_below();
    test::cleanup_unused_pages_three_above();
    // merge pages
    test::merge_pages();

    let page1 = get_page(0);

    // insert an entry on single-page
    test::insert_entry_empty_page(page1);
    test::insert_entry(page1);
    test::insert_entry_full_page(page1);
    // remove an entry on single-page
    test::remove_entry(page1);
    test::fail__remove_entry_unexisting(page1);
    // take an entry on single-page
    test::take_entry(page1);
    test::fail__take_entry_unexisting(page1);
    // drop an entry on single-page
    test::drop_entry(page1);
    test::fail__drop_entry_unexisting(page1);
    // update an entry on single-page
    test::update_entry_resize_smaller(page1);
    test::update_entry_resize_bigger(page1);
    test::fail__update_entry_resize_unexisting(page1);
    // query on single-page
    test::query_relocation_inplace_smaller(page1);
    test::query_relocation_inplace_bigger_contiguous_free_bigger(page1);
    test::query_relocation_inplace_bigger_contiguous_free_smaller(page1);
    test::query_relocation_inplace_bigger_noncontiguous_free(page1);
    test::query_relocation_inplace_bigger_contiguous_taken(page1);
    test::fail__query_relocation_inplace_unexisting(page1);

    add_second_page();
    let page2 = get_page(1);

    // insert an entry on multi-page
    test::insert_entry_first_page(page1, page2);
    test::insert_entry_second_page_empty(page1, page2);
    test::insert_entry_second_page_free(page1, page2);
    test::insert_entry_second_page_taken(page1, page2);
    test::insert_entry_second_page_multiple(page1, page2);
    // remove an entry on multi-page
    test::remove_entry_first_page(page1, page2);
    test::remove_entry_partially_first_page(page1, page2);
    test::remove_entry_first_page_last_with_reminder(page1, page2);
    test::remove_entry_second_page(page1, page2);
    // take an entry on multi-page
    test::take_entry_partially_first_page(page1, page2);
    test::take_entry_entirely_second_page(page1, page2);
    // drop an entry on multi-page
    test::drop_entry_first_page(page1, page2);
    test::drop_entry_second_page(page1, page2);
    // update an entry on multi-page
    test::update_entry_resize_smaller_first_page(page1, page2);
    test::update_entry_resize_bigger_first_page(page1, page2);
    test::update_entry_resize_smaller_second_page(page1, page2);
    //query on multi-page
    test::query_relocation_inplace_first_page(page1, page2);
    test::query_relocation_inplace_second_page(page1, page2);

    remove_second_page();
    clear_page(page1);
}

mod test {

use super::*;
use crate::memory::paging::tracing::tests::count_tracing_pages;
use crate::traits::*;

pub(super) const
OWNER : MemoryOwner = MemoryOwner::Kernel;

/// # Test
///
/// ### Given
///
/// - A tracing table with one [`TracingPage`]
///
/// ### When
///
/// - Requesting to create a new [`TracingPage`]
///
/// ### Then
///
/// - The new page is created correctly
pub(super) fn create_new_page() {
    scenario!("Create new tracing page");
    // Given
    assume!(count_tracing_pages() == 1, "tracing pre-conditions");
    // When / Then
    check!(create::create_tracing_page(OWNER).is_ok(), "failed to create");
    // Then
    assume!(count_tracing_pages() == 2, "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A tracing table with two [`TracingPage`]
///
/// ### When
///
/// - Requesting to delete the second [`TracingPage`]
///
/// ### Then
///
/// - The page is deleted correctly
pub(super) fn delete_page() {
    scenario!("Delete tracing page");
    // Given
    assume!(count_tracing_pages() == 2, "tracing pre-conditions");
    // When / Then
    check!(delete::delete_tracing_page(get_page_paddr(1), OWNER).is_ok(), "failed to delete");
    // Then
    assume!(count_tracing_pages() == 1, "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A tracing table with two [`TracingPage`], both empty
///
/// ### When
///
/// - Requesting to clean-up the unused pages
///
/// ### Then
///
/// - The first page is kept
/// - The second page is deleted
pub(super) fn cleanup_unused_pages_two_empty() {
    scenario!("Clean-up unused pages (2, empty)");
    // Given
    let _ = create::create_tracing_page(OWNER).get_or_panic();
    assume!(count_tracing_pages() == 2, "tracing pre-conditions");
    assume!(get_page(0).is_empty(), "page pre-conditions");
    assume!(get_page(1).is_empty(), "page pre-conditions");
    // When / Then
    check!(clean::cleanup_unused_tracing_pages(OWNER).is_ok(), "failed to clean");
    // Then
    assume!(count_tracing_pages() == 1, "tracing post-conditions");
    assume!(get_page(0).is_empty(), "page post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A tracing table with two [`TracingPage`], of which the first contain
///   less entries than the size threshold and the second is empty
///
/// ### When
///
/// - Requesting to clean-up the unused pages
///
/// ### Then
///
/// - The first page is kept
/// - The second page is deleted
pub(super) fn cleanup_unused_pages_two_below() {
    scenario!("Clean-up unused pages (2, below)");
    // Given
    let _ = create::create_tracing_page(OWNER).get_or_panic();
    assume!(count_tracing_pages() == 2, "tracing pre-conditions");
    clear_page(get_page(0));
    add_entries(get_page(0));
    assume!(get_page(0).size() == 4, "page pre-conditions");
    assume!(get_page(1).is_empty(), "page pre-conditions");
    // When / Then
    check!(clean::cleanup_unused_tracing_pages(OWNER).is_ok(), "failed to clean");
    // Then
    assume!(count_tracing_pages() == 1, "tracing post-conditions");
    assume!(get_page(0).size() == 4, "page post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A tracing table with two [`TracingPage`], of which the first contain
///   more entries than the size threshold and the second is empty
///
/// ### When
///
/// - Requesting to clean-up the unused pages
///
/// ### Then
///
/// - The first page is kept
/// - The second page is kept
pub(super) fn cleanup_unused_pages_two_above() {
    scenario!("Clean-up unused pages (2, above)");
    // Given
    let _ = create::create_tracing_page(OWNER).get_or_panic();
    assume!(count_tracing_pages() == 2, "tracing pre-conditions");
    clear_page(get_page(0));
    fill_page_taken(get_page(0));
    assume!(get_page(0).is_full(), "page pre-conditions");
    assume!(get_page(1).is_empty(), "page pre-conditions");
    // When / Then
    check!(clean::cleanup_unused_tracing_pages(OWNER).is_ok(), "failed to clean");
    // Then
    assume!(count_tracing_pages() == 2, "tracing post-conditions");
    assume!(get_page(0).is_full(), "page post-conditions");
    assume!(get_page(1).is_empty(), "page post-conditions");
    delete::delete_tracing_page(get_page_paddr(1), OWNER).ok_or_panic();
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A tracing table with three [`TracingPage`], of which the first contain
///   less entries than the size threshold and the others are empty
///
/// ### When
///
/// - Requesting to clean-up the unused pages
///
/// ### Then
///
/// - The first page is kept
/// - The second and third page are deleted
pub(super) fn cleanup_unused_pages_three_below() {
    scenario!("Clean-up unused pages (3, below)");
    // Given
    let _ = create::create_tracing_page(OWNER).get_or_panic();
    let _ = create::create_tracing_page(OWNER).get_or_panic();
    assume!(count_tracing_pages() == 3, "tracing pre-conditions");
    clear_page(get_page(0));
    add_entries(get_page(0));
    assume!(get_page(0).size() == 4, "page pre-conditions");
    assume!(get_page(1).is_empty(), "page pre-conditions");
    assume!(get_page(2).is_empty(), "page pre-conditions");
    // When / Then
    check!(clean::cleanup_unused_tracing_pages(OWNER).is_ok(), "failed to clean");
    // Then
    assume!(count_tracing_pages() == 1, "tracing post-conditions");
    assume!(get_page(0).size() == 4, "page post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A tracing table with three [`TracingPage`], of which the first contain
///   more entries than the size threshold and the others are empty
///
/// ### When
///
/// - Requesting to clean-up the unused pages
///
/// ### Then
///
/// - The first and second pages are kept
/// - The third page is deleted
pub(super) fn cleanup_unused_pages_three_above() {
    scenario!("Clean-up unused pages (3, above)");
    // Given
    let _ = create::create_tracing_page(OWNER).get_or_panic();
    let _ = create::create_tracing_page(OWNER).get_or_panic();
    assume!(count_tracing_pages() == 3, "tracing pre-conditions");
    clear_page(get_page(0));
    fill_page_taken(get_page(0));
    assume!(get_page(0).is_full(), "page pre-conditions");
    assume!(get_page(1).is_empty(), "page pre-conditions");
    assume!(get_page(2).is_empty(), "page pre-conditions");
    // When / Then
    check!(clean::cleanup_unused_tracing_pages(OWNER).is_ok(), "failed to clean");
    // Then
    assume!(count_tracing_pages() == 2, "tracing post-conditions");
    assume!(get_page(0).is_full(), "page post-conditions");
    assume!(get_page(1).is_empty(), "page post-conditions");
    delete::delete_tracing_page(get_page_paddr(1), OWNER).ok_or_panic();
    test_passed!();
    wait!();
}


/// # Test
///
/// ### Given
///
/// - A tracing table with two [`TracingPage`], both containing some entries
///   but not full
///
/// ### When
///
/// - Requesting to merge the pages
///
/// ### Then
///
/// - The antries of the second pages are moved to the first page correctly
/// - The second page is not deleted
pub(super) fn merge_pages() {
    scenario!("Merge pages");
    // Given
    let _ = create::create_tracing_page(OWNER).get_or_panic();
    assume!(count_tracing_pages() == 2, "tracing pre-conditions");
    let first_page = get_page(0);
    clear_page(first_page);
    first_page.append_unchecked(Metadata::new_taken(PhysicalAddress::from(0x1000), LogicalAddress::from(0xA000), 0x1000));
    first_page.append_unchecked(Metadata::new_taken(PhysicalAddress::from(0x2000), LogicalAddress::from(0xB000), 0x1000));
    first_page.append_unchecked(Metadata::new_free(PhysicalAddress::from(0x3000), LogicalAddress::from(0xC000), 0x1000));
    assume!(first_page.size() == 3, "page pre-conditions");
    let second_page = get_page(1);
    clear_page(second_page);
    second_page.append_unchecked(Metadata::new_free(PhysicalAddress::from(0x4000), LogicalAddress::from(0xD000), 0x1000));
    second_page.append_unchecked(Metadata::new_taken(PhysicalAddress::from(0x5000), LogicalAddress::from(0xE000), 0x1000));
    assume!(second_page.size() == 2, "page pre-conditions");
    // When / Then
    check!(merge::merge_tracing_pages(OWNER).is_ok(), "failed to merge");
    // Then
    check!(count_tracing_pages() == 2, "pages count is different");
    check!(first_page.size() == 4, "first page size is different");
    check!(second_page.is_empty(), "second page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000), LogicalAddress::from(0xA000), 0x1000);
    check!(compare_metadata(&first_page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x2000), LogicalAddress::from(0xB000), 0x1000);
    check!(compare_metadata(&first_page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x3000), LogicalAddress::from(0xC000), 0x2000);
    check!(compare_metadata(&first_page.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x5000), LogicalAddress::from(0xE000), 0x1000);
    check!(compare_metadata(&first_page.entry_at(3), &expected_md), "entry 3 is different");
    delete::delete_tracing_page(get_page_paddr(1), OWNER).ok_or_panic();
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A tracing table with one empty [`TracingPage`]
///
/// ### When
///
/// - Requesting to insert an entry
///
/// ### Then
///
/// - The entry is inserted successfully
/// - No new page is created
pub(super) fn insert_entry_empty_page(page1:&mut TracingPage) {
    scenario!("Insert an entry in an empty page");
    // Given
    clear_page(page1);
    assume!(page1.is_empty() == true, "page pre-conditions");
    assume!(count_tracing_pages() == 1, "tracing pre-conditions");
    // When / Then
    let new_md = Metadata::new_taken(PhysicalAddress::from(0x1000), LogicalAddress::from(0xA000), 0x100);
    check!(insert::insert_entry(new_md, OWNER).is_ok(), "failed to insert");
    // Then
    check!(page1.size() == 1, "page size is different");
    check!(compare_metadata(&page1.entry_at(0), &new_md), "first entry is different");
    assume!(count_tracing_pages() == 1, "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A tracing table with one [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to insert an entry
///
/// ### Then
///
/// - The entry is inserted successfully
/// - No new page is created
pub(super) fn insert_entry(page1:&mut TracingPage) {
    scenario!("Insert an entry in a page");
    // Given
    clear_page(page1);
    add_entries(page1);
    assume!(page1.size() == 4, "page pre-conditions");
    assume!(count_tracing_pages() == 1, "tracing pre-conditions");
    // When / Then
    let new_md = Metadata::new_taken(PhysicalAddress::from(0x2800), LogicalAddress::from(0xB800), 0x100);
    check!(insert::insert_entry(new_md, OWNER).is_ok(), "failed to insert");
    // Then
    check!(page1.size() == 5, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000), LogicalAddress::from(0xA000), 0x100);
    check!(compare_metadata(&page1.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x2000), LogicalAddress::from(0xB000), 0x100);
    check!(compare_metadata(&page1.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = new_md;
    check!(compare_metadata(&page1.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x3000), LogicalAddress::from(0xC000), 0x100);
    check!(compare_metadata(&page1.entry_at(3), &expected_md), "entry 3 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x4000), LogicalAddress::from(0xD000), 0x100);
    check!(compare_metadata(&page1.entry_at(4), &expected_md), "entry 4 is different");
    assume!(count_tracing_pages() == 1, "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A tracing table with one full [`TracingPage`]
///
/// ### When
///
/// - Requesting to insert an entry
///
/// ### Then
///
/// - The entry is inserted successfully
/// - The page is updated correctly
/// - A new page is created
/// - The exceeding entry is stored in the new page
pub(super) fn insert_entry_full_page(page1:&mut TracingPage) {
    scenario!("Insert an entry in a full page");
    // Given
    clear_page(page1);
    fill_page_taken(page1);
    assume!(page1.is_full(), "page pre-conditions");
    assume!(count_tracing_pages() == 1, "tracing pre-conditions");
    // When / Then
    let new_md = Metadata::new_taken(PhysicalAddress::from(0xF00), LogicalAddress::from(0x9F00), 0x100);
    check!(insert::insert_entry(new_md, OWNER).is_ok(), "failed to insert");
    // Then
    check!(count_tracing_pages() == 2, "pages count is different");
    check!(page1.is_full(), "first page size is different");
    let expected_md = new_md;
    check!(compare_metadata(&page1.entry_at(0), &expected_md), "first entry of first page is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000D00), LogicalAddress::from(0x1009D00), 0x100);
    check!(compare_metadata(&page1.entry_at(METADATA_ARRAY_SIZE-1), &expected_md), "last entry of first page is different");
    let page2 = get_page(1);
    check!(page2.size() == 1, "second page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000E00), LogicalAddress::from(0x1009E00), 0x100);
    check!(compare_metadata(&page2.entry_at(0), &expected_md), "first entry of second page is different");
    delete::delete_tracing_page(get_page_paddr(1), OWNER).ok_or_panic();
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A tracing table with one [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to remove an entry
///
/// ### Then
///
/// - The entry is removed successfully
/// - The page is updated correctly
/// - No new page is created
pub(super) fn remove_entry(page1:&mut TracingPage) {
    scenario!("Remove an entry from a page");
    // Given
    clear_page(page1);
    add_entries(page1);
    assume!(page1.size() == 4, "page pre-conditions");
    assume!(count_tracing_pages() == 1, "tracing pre-conditions");
    // When / Then
    check!(remove::remove_space(0x3000.into(), 0x100, OWNER).is_ok(), "failed to remove");
    // Then
    check!(page1.size() == 3, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000), LogicalAddress::from(0xA000), 0x100);
    check!(compare_metadata(&page1.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x2000), LogicalAddress::from(0xB000), 0x100);
    check!(compare_metadata(&page1.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x4000), LogicalAddress::from(0xD000), 0x100);
    check!(compare_metadata(&page1.entry_at(2), &expected_md), "entry 2 is different");
    assume!(count_tracing_pages() == 1, "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A tracing table with one [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to remove an entry which does not exist in the page
///
/// ### Then
///
/// - The request is refused
/// - The page is not modified
/// - No new page is created
pub(super) fn fail__remove_entry_unexisting(page1:&mut TracingPage) {
    fail_scenario!("Remove an unexisting entry from a page");
    // Given
    clear_page(page1);
    add_entries(page1);
    assume!(page1.size() == 4, "page pre-conditions");
    assume!(count_tracing_pages() == 1, "tracing pre-conditions");
    // When / Then
    check!(remove::remove_space(0x5000.into(), 0x100, OWNER).is_err(), "result is Ok");
    // Then
    check!(page1.size() == 4, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000), LogicalAddress::from(0xA000), 0x100);
    check!(compare_metadata(&page1.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x2000), LogicalAddress::from(0xB000), 0x100);
    check!(compare_metadata(&page1.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x3000), LogicalAddress::from(0xC000), 0x100);
    check!(compare_metadata(&page1.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x4000), LogicalAddress::from(0xD000), 0x100);
    check!(compare_metadata(&page1.entry_at(3), &expected_md), "entry 3 is different");
    assume!(count_tracing_pages() == 1, "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A tracing table with one [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to take a free entry
///
/// ### Then
///
/// - The entry is updated correctly
/// - No new page is created
pub(super) fn take_entry(page1:&mut TracingPage) {
    scenario!("Take an entry from a page");
    // Given
    clear_page(page1);
    add_entries(page1);
    assume!(page1.size() == 4, "page pre-conditions");
    assume!(count_tracing_pages() == 1, "tracing pre-conditions");
    // When / Then
    check!(take::take_available_space(0x3000.into(), 0x100, OWNER).is_ok(), "failed to take");
    // Then
    check!(page1.size() == 4, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000), LogicalAddress::from(0xA000), 0x100);
    check!(compare_metadata(&page1.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x2000), LogicalAddress::from(0xB000), 0x100);
    check!(compare_metadata(&page1.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x3000), LogicalAddress::from(0xC000), 0x100);
    check!(compare_metadata(&page1.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x4000), LogicalAddress::from(0xD000), 0x100);
    check!(compare_metadata(&page1.entry_at(3), &expected_md), "entry 3 is different");
    assume!(count_tracing_pages() == 1, "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A tracing table with one [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to take a free entry which does not exist in the page
///
/// ### Then
///
/// - The request is refused
/// - The page is not modified
/// - No new page is created
pub(super) fn fail__take_entry_unexisting(page1:&mut TracingPage) {
    fail_scenario!("Take an unexisting entry from a page");
    // Given
    clear_page(page1);
    add_entries(page1);
    assume!(page1.size() == 4, "page pre-conditions");
    assume!(count_tracing_pages() == 1, "tracing pre-conditions");
    // When / Then
    check!(take::take_available_space(0x5000.into(), 0x100, OWNER).is_err(), "result is Ok");
    // Then
    check!(page1.size() == 4, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000), LogicalAddress::from(0xA000), 0x100);
    check!(compare_metadata(&page1.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x2000), LogicalAddress::from(0xB000), 0x100);
    check!(compare_metadata(&page1.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x3000), LogicalAddress::from(0xC000), 0x100);
    check!(compare_metadata(&page1.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x4000), LogicalAddress::from(0xD000), 0x100);
    check!(compare_metadata(&page1.entry_at(3), &expected_md), "entry 3 is different");
    assume!(count_tracing_pages() == 1, "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A tracing table with one [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to drop a taken entry
///
/// ### Then
///
/// - The entry is updated correctly
/// - No new page is created
pub(super) fn drop_entry(page1:&mut TracingPage) {
    scenario!("Drop an entry from a page");
    // Given
    clear_page(page1);
    add_entries(page1);
    assume!(page1.size() == 4, "page pre-conditions");
    assume!(count_tracing_pages() == 1, "tracing pre-conditions");
    // When / Then
    check!(drop::drop_occupied_space(0x4000.into(), OWNER).is_ok(), "failed to take");
    // Then
    check!(page1.size() == 4, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000), LogicalAddress::from(0xA000), 0x100);
    check!(compare_metadata(&page1.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x2000), LogicalAddress::from(0xB000), 0x100);
    check!(compare_metadata(&page1.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x3000), LogicalAddress::from(0xC000), 0x100);
    check!(compare_metadata(&page1.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x4000), LogicalAddress::from(0xD000), 0x100);
    check!(compare_metadata(&page1.entry_at(3), &expected_md), "entry 3 is different");
    assume!(count_tracing_pages() == 1, "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A tracing table with one [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to drop a taken entry which does not exist in the page
///
/// ### Then
///
/// - The request is refused
/// - The page is not modified
/// - No new page is created
pub(super) fn fail__drop_entry_unexisting(page1:&mut TracingPage) {
    fail_scenario!("Drop an unexisting entry from a page");
    // Given
    clear_page(page1);
    add_entries(page1);
    assume!(page1.size() == 4, "page pre-conditions");
    assume!(count_tracing_pages() == 1, "tracing pre-conditions");
    // When / Then
    check!(drop::drop_occupied_space(0x5000.into(), OWNER).is_err(), "result is Ok");
    // Then
    check!(page1.size() == 4, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000), LogicalAddress::from(0xA000), 0x100);
    check!(compare_metadata(&page1.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x2000), LogicalAddress::from(0xB000), 0x100);
    check!(compare_metadata(&page1.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x3000), LogicalAddress::from(0xC000), 0x100);
    check!(compare_metadata(&page1.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x4000), LogicalAddress::from(0xD000), 0x100);
    check!(compare_metadata(&page1.entry_at(3), &expected_md), "entry 3 is different");
    assume!(count_tracing_pages() == 1, "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A tracing table with one [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to resize a taken entry to a smaller size
///
/// ### Then
///
/// - The entry is resized successfully
/// - The page is updated correctly
/// - No new page is created
pub(super) fn update_entry_resize_smaller(page1:&mut TracingPage) {
    scenario!("Resize an entry from a page to a smaller size");
    // Given
    clear_page(page1);
    add_entries(page1);
    assume!(page1.size() == 4, "page pre-conditions");
    assume!(count_tracing_pages() == 1, "tracing pre-conditions");
    // When / Then
    check!(update::resize(0x4000.into(), 0x80, OWNER).is_ok(), "failed to resize");
    // Then
    check!(page1.size() == 5, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000), LogicalAddress::from(0xA000), 0x100);
    check!(compare_metadata(&page1.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x2000), LogicalAddress::from(0xB000), 0x100);
    check!(compare_metadata(&page1.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x3000), LogicalAddress::from(0xC000), 0x100);
    check!(compare_metadata(&page1.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x4000), LogicalAddress::from(0xD000), 0x80);
    check!(compare_metadata(&page1.entry_at(3), &expected_md), "entry 3 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x4080), LogicalAddress::from(0xD080), 0x80);
    check!(compare_metadata(&page1.entry_at(4), &expected_md), "entry 4 is different");
    assume!(count_tracing_pages() == 1, "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A tracing table with one [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to resize a taken entry to a bigger size
///
/// ### Then
///
/// - The entry is resized successfully
/// - The page is updated correctly
/// - No new page is created
pub(super) fn update_entry_resize_bigger(page1:&mut TracingPage) {
    scenario!("Resize an entry from a page to a bigger size");
    // Given
    clear_page(page1);
    add_entries(page1);
    *page1.entry_at_mut(0) = Metadata::new_taken(PhysicalAddress::from(0x1000), LogicalAddress::from(0xA000), 0x1000);
    assume!(page1.size() == 4, "page pre-conditions");
    assume!(count_tracing_pages() == 1, "tracing pre-conditions");
    // When / Then
    check!(update::resize(0x1000.into(), 0x1100, OWNER).is_ok(), "failed to resize");
    // Then
    check!(page1.size() == 3, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000), LogicalAddress::from(0xA000), 0x1100);
    check!(compare_metadata(&page1.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x3000), LogicalAddress::from(0xC000), 0x100);
    check!(compare_metadata(&page1.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x4000), LogicalAddress::from(0xD000), 0x100);
    check!(compare_metadata(&page1.entry_at(2), &expected_md), "entry 2 is different");
    assume!(count_tracing_pages() == 1, "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A tracing table with one [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to resize a taken entry which does not exist in the page
///
/// ### Then
///
/// - The request is refused
/// - The page is not modified
/// - No new page is created
pub(super) fn fail__update_entry_resize_unexisting(page1:&mut TracingPage) {
    fail_scenario!("Resize an unexisting entry from a page");
    // Given
    clear_page(page1);
    add_entries(page1);
    assume!(page1.size() == 4, "page pre-conditions");
    assume!(count_tracing_pages() == 1, "tracing pre-conditions");
    // When / Then
    check!(update::resize(0x5000.into(), 0x100, OWNER).is_err(), "result is Ok");
    // Then
    check!(page1.size() == 4, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000), LogicalAddress::from(0xA000), 0x100);
    check!(compare_metadata(&page1.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x2000), LogicalAddress::from(0xB000), 0x100);
    check!(compare_metadata(&page1.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x3000), LogicalAddress::from(0xC000), 0x100);
    check!(compare_metadata(&page1.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x4000), LogicalAddress::from(0xD000), 0x100);
    check!(compare_metadata(&page1.entry_at(3), &expected_md), "entry 3 is different");
    assume!(count_tracing_pages() == 1, "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A tracing table with one [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to check whether a taken entry can be resized to a smaller size
///
/// ### Then
///
/// - The request is processed successfully
/// - The page is not modified
/// - No new page is created
pub(super) fn query_relocation_inplace_smaller(page1:&mut TracingPage) {
    scenario!("Query relocation inplace to a smaller size on a page");
    // Given
    clear_page(page1);
    add_entries(page1);
    assume!(page1.size() == 4, "page pre-conditions");
    assume!(count_tracing_pages() == 1, "tracing pre-conditions");
    // When
    let query_result = query::can_relocate_inplace(0xD000.into(), 0x80, OWNER);
    // Then
    check!(query_result.is_ok(), "failed to query");
    check!(query_result.get_or_panic().0 == true, "query result is different");
    check!(page1.size() == 4, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000), LogicalAddress::from(0xA000), 0x100);
    check!(compare_metadata(&page1.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x2000), LogicalAddress::from(0xB000), 0x100);
    check!(compare_metadata(&page1.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x3000), LogicalAddress::from(0xC000), 0x100);
    check!(compare_metadata(&page1.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x4000), LogicalAddress::from(0xD000), 0x100);
    check!(compare_metadata(&page1.entry_at(3), &expected_md), "entry 3 is different");
    assume!(count_tracing_pages() == 1, "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A tracing table with one [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to check whether a taken entry, which is contiguous to a free entry
///   with enough space, can be resized to a bigger size
///
/// ### Then
///
/// - The request is processed successfully
/// - The page is not modified
/// - No new page is created
pub(super) fn query_relocation_inplace_bigger_contiguous_free_bigger(page1:&mut TracingPage) {
    scenario!("Query relocation inplace to a bigger size on a page (contiguous free bigger)");
    // Given
    clear_page(page1);
    add_entries(page1);
    *page1.entry_at_mut(0) = Metadata::new_taken(PhysicalAddress::from(0x1000), LogicalAddress::from(0xA000), 0x1000);
    assume!(page1.size() == 4, "page pre-conditions");
    assume!(count_tracing_pages() == 1, "tracing pre-conditions");
    // When
    let query_result = query::can_relocate_inplace(0xA000.into(), 0x1080, OWNER);
    // Then
    check!(query_result.is_ok(), "failed to query");
    check!(query_result.get_or_panic().0 == true, "query result is different");
    check!(page1.size() == 4, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000), LogicalAddress::from(0xA000), 0x1000);
    check!(compare_metadata(&page1.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x2000), LogicalAddress::from(0xB000), 0x100);
    check!(compare_metadata(&page1.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x3000), LogicalAddress::from(0xC000), 0x100);
    check!(compare_metadata(&page1.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x4000), LogicalAddress::from(0xD000), 0x100);
    check!(compare_metadata(&page1.entry_at(3), &expected_md), "entry 3 is different");
    assume!(count_tracing_pages() == 1, "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A tracing table with one [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to check whether a taken entry, which is contiguous to a free entry
///   without enough space, can be resized to a bigger size
///
/// ### Then
///
/// - The request is processed successfully
/// - The page is not modified
/// - No new page is created
pub(super) fn query_relocation_inplace_bigger_contiguous_free_smaller(page1:&mut TracingPage) {
    scenario!("Query relocation inplace to a bigger size on a page (contiguous free smaller)");
    // Given
    clear_page(page1);
    add_entries(page1);
    *page1.entry_at_mut(0) = Metadata::new_taken(PhysicalAddress::from(0x1000), LogicalAddress::from(0xA000), 0x1000);
    assume!(page1.size() == 4, "page pre-conditions");
    assume!(count_tracing_pages() == 1, "tracing pre-conditions");
    // When
    let query_result = query::can_relocate_inplace(0xA000.into(), 0x1200, OWNER);
    // Then
    check!(query_result.is_ok(), "failed to query");
    check!(query_result.get_or_panic().0 == false, "query result is different");
    check!(page1.size() == 4, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000), LogicalAddress::from(0xA000), 0x1000);
    check!(compare_metadata(&page1.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x2000), LogicalAddress::from(0xB000), 0x100);
    check!(compare_metadata(&page1.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x3000), LogicalAddress::from(0xC000), 0x100);
    check!(compare_metadata(&page1.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x4000), LogicalAddress::from(0xD000), 0x100);
    check!(compare_metadata(&page1.entry_at(3), &expected_md), "entry 3 is different");
    assume!(count_tracing_pages() == 1, "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A tracing table with one [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to check whether a taken entry, which is stored before a free entry
///   but is not contiguous to it, can be resized to a bigger size
///
/// ### Then
///
/// - The request is processed successfully
/// - The page is not modified
/// - No new page is created
pub(super) fn query_relocation_inplace_bigger_noncontiguous_free(page1:&mut TracingPage) {
    scenario!("Query relocation inplace to a bigger size on a page (non-contiguous free)");
    // Given
    clear_page(page1);
    add_entries(page1);
    assume!(page1.size() == 4, "page pre-conditions");
    assume!(count_tracing_pages() == 1, "tracing pre-conditions");
    // When
    let query_result = query::can_relocate_inplace(0xA000.into(), 0x1080, OWNER);
    // Then
    check!(query_result.is_ok(), "failed to query");
    check!(query_result.get_or_panic().0 == false, "query result is different");
    check!(page1.size() == 4, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000), LogicalAddress::from(0xA000), 0x100);
    check!(compare_metadata(&page1.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x2000), LogicalAddress::from(0xB000), 0x100);
    check!(compare_metadata(&page1.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x3000), LogicalAddress::from(0xC000), 0x100);
    check!(compare_metadata(&page1.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x4000), LogicalAddress::from(0xD000), 0x100);
    check!(compare_metadata(&page1.entry_at(3), &expected_md), "entry 3 is different");
    assume!(count_tracing_pages() == 1, "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A tracing table with one [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to check whether a taken entry, which is contiguous to a taken
///   entry, can be resized to a bigger size
///
/// ### Then
///
/// - The request is processed successfully
/// - The page is not modified
/// - No new page is created
pub(super) fn query_relocation_inplace_bigger_contiguous_taken(page1:&mut TracingPage) {
    scenario!("Query relocation inplace to a bigger size on a page (contiguous taken)");
    // Given
    clear_page(page1);
    add_entries(page1);
    *page1.entry_at_mut(0) = Metadata::new_taken(PhysicalAddress::from(0x1000), LogicalAddress::from(0xA000), 0x1000);
    page1.entry_at_mut(1).set_taken();
    assume!(page1.size() == 4, "page pre-conditions");
    assume!(count_tracing_pages() == 1, "tracing pre-conditions");
    // When
    let query_result = query::can_relocate_inplace(0xA000.into(), 0x1080, OWNER);
    // Then
    check!(query_result.is_ok(), "failed to query");
    check!(query_result.get_or_panic().0 == false, "query result is different");
    check!(page1.size() == 4, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000), LogicalAddress::from(0xA000), 0x1000);
    check!(compare_metadata(&page1.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x2000), LogicalAddress::from(0xB000), 0x100);
    check!(compare_metadata(&page1.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x3000), LogicalAddress::from(0xC000), 0x100);
    check!(compare_metadata(&page1.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x4000), LogicalAddress::from(0xD000), 0x100);
    check!(compare_metadata(&page1.entry_at(3), &expected_md), "entry 3 is different");
    assume!(count_tracing_pages() == 1, "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A tracing table with one [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to check whether a taken entry, which does not exist in the page,
///   can be resized
///
/// ### Then
///
/// - The request is refused
/// - The page is not modified
/// - No new page is created
pub(super) fn fail__query_relocation_inplace_unexisting(page1:&mut TracingPage) {
    fail_scenario!("Query relocation inplace of an unexisting entry of a page");
    // Given
    clear_page(page1);
    add_entries(page1);
    assume!(page1.size() == 4, "page pre-conditions");
    assume!(count_tracing_pages() == 1, "tracing pre-conditions");
    // When / Then
    check!(query::can_relocate_inplace(0xE000.into(), 0x80, OWNER).is_err(), "result is Ok");
    // Then
    check!(page1.size() == 4, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000), LogicalAddress::from(0xA000), 0x100);
    check!(compare_metadata(&page1.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x2000), LogicalAddress::from(0xB000), 0x100);
    check!(compare_metadata(&page1.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x3000), LogicalAddress::from(0xC000), 0x100);
    check!(compare_metadata(&page1.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x4000), LogicalAddress::from(0xD000), 0x100);
    check!(compare_metadata(&page1.entry_at(3), &expected_md), "entry 3 is different");
    assume!(count_tracing_pages() == 1, "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A tracing table with two [`TracingPage`], of which the first is full and
///   the second contains some entries
///
/// ### When
///
/// - Requesting to insert an entry having an address in the range of addresses
///   of the first page
///
/// ### Then
///
/// - The entry is inserted successfully in the first page
/// - The first page is updated correctly
/// - The exceeding entry from the first page is moved to the second page
/// - The second page is updated correctly
/// - No new page is created
pub(super) fn insert_entry_first_page(page1:&mut TracingPage, page2:&mut TracingPage) {
    scenario!("Insert an entry (multi-page, first)");
    // Given
    assume!(count_tracing_pages() == 2, "tracing pre-conditions");
    clear_page(page1);
    fill_page_taken(page1);
    assume!(page1.is_full(), "page1 pre-conditions");
    clear_page(page2);
    add_multiple_entries(page2);
    assume!(page2.size() == 4, "page2 pre-conditions");
    // When / Then
    let new_md = Metadata::new_taken(PhysicalAddress::from(0xF00), LogicalAddress::from(0x9F00), 0x100);
    check!(insert::insert_entry(new_md, OWNER).is_ok(), "failed to insert");
    // Then
    check!(page1.is_full(), "page 1 size is different");
    let expected_md = new_md;
    check!(compare_metadata(&page1.entry_at(0), &expected_md), "first entry of page 1 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000D00), LogicalAddress::from(0x1009D00), 0x100);
    check!(compare_metadata(&page1.entry_at(METADATA_ARRAY_SIZE-1), &expected_md), "last entry of page 1 is different");
    check!(page2.size() == 5, "page 2 size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000E00), LogicalAddress::from(0x1009E00), 0x100);
    check!(compare_metadata(&page2.entry_at(0), &expected_md), "first entry of page 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1001400), LogicalAddress::from(0x100A400), 0x100);
    check!(compare_metadata(&page2.entry_at(4), &expected_md), "last entry of page 2 is different");
    assume!(count_tracing_pages() == 2, "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A tracing table with two [`TracingPage`], of which the first is full and
///   the second is empty
///
/// ### When
///
/// - Requesting to insert an entry which is contiguous to the last entry of
///   the first page
///
/// ### Then
///
/// - The entry is inserted successfully in the second page
/// - The first page is not modified
/// - No new page is created
pub(super) fn insert_entry_second_page_empty(page1:&mut TracingPage, page2:&mut TracingPage) {
    scenario!("Insert an entry (multi-page, second empty)");
    // Given
    assume!(count_tracing_pages() == 2, "tracing pre-conditions");
    clear_page(page1);
    fill_page_taken(page1);
    assume!(page1.is_full(), "page1 pre-conditions");
    clear_page(page2);
    assume!(page2.is_empty(), "page2 pre-conditions");
    // When / Then
    let new_md = Metadata::new_taken(PhysicalAddress::from(0x1000F00), LogicalAddress::from(0x1009F00), 0x100);
    check!(insert::insert_entry(new_md, OWNER).is_ok(), "failed to insert");
    // Then
    check!(page1.is_full(), "page 1 size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000E00), LogicalAddress::from(0x1009E00), 0x100);
    check!(compare_metadata(&page1.entry_at(METADATA_ARRAY_SIZE-1), &expected_md), "last entry of page 1 is different");
    check!(page2.size() == 1, "page 2 size is different");
    let expected_md = new_md;
    check!(compare_metadata(&page2.entry_at(0), &expected_md), "first entry of page 2 is different");
    assume!(count_tracing_pages() == 2, "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A tracing table with two [`TracingPage`], of which the first is full and
///   the second contains one free entry
///
/// ### When
///
/// - Requesting to insert a free entry which is contiguous to the first entry of
///   the second page
///
/// ### Then
///
/// - The entry is merged with the entry in the second page
/// - The first page is not modified
/// - No new page is created
pub(super) fn insert_entry_second_page_free(page1:&mut TracingPage, page2:&mut TracingPage) {
    scenario!("Insert an entry (multi-page, second free)");
    // Given
    assume!(count_tracing_pages() == 2, "tracing pre-conditions");
    clear_page(page1);
    fill_page_taken(page1);
    assume!(page1.is_full(), "page1 pre-conditions");
    clear_page(page2);
    add_contiguous_free_entry(page2);
    assume!(page2.size() == 1, "page2 pre-conditions");
    // When / Then
    let new_md = Metadata::new_free(PhysicalAddress::from(0x1001000), LogicalAddress::from(0x100A000), 0x100);
    check!(insert::insert_entry(new_md, OWNER).is_ok(), "failed to insert");
    // Then
    check!(page1.is_full(), "page 1 size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000E00), LogicalAddress::from(0x1009E00), 0x100);
    check!(compare_metadata(&page1.entry_at(METADATA_ARRAY_SIZE-1), &expected_md), "last entry of page 1 is different");
    check!(page2.size() == 1, "page 2 size is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1000F00), LogicalAddress::from(0x1009F00), 0x200);
    check!(compare_metadata(&page2.entry_at(0), &expected_md), "first entry of page 2 is different");
    assume!(count_tracing_pages() == 2, "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A tracing table with two [`TracingPage`], of which the first is full and
///   the second contains one taken entry
///
/// ### When
///
/// - Requesting to insert an entry which is contiguous to the first entry of
///   the second page
///
/// ### Then
///
/// - The entry is inserted successfully in the second page
/// - The first page is not modified
/// - No new page is created
pub(super) fn insert_entry_second_page_taken(page1:&mut TracingPage, page2:&mut TracingPage) {
    scenario!("Insert an entry (multi-page, second taken)");
    // Given
    assume!(count_tracing_pages() == 2, "tracing pre-conditions");
    clear_page(page1);
    fill_page_taken(page1);
    assume!(page1.is_full(), "page1 pre-conditions");
    clear_page(page2);
    add_contiguous_taken_entry(page2);
    assume!(page2.size() == 1, "page2 pre-conditions");
    // When / Then
    let new_md = Metadata::new_taken(PhysicalAddress::from(0x1001000), LogicalAddress::from(0x100A000), 0x100);
    check!(insert::insert_entry(new_md, OWNER).is_ok(), "failed to insert");
    // Then
    check!(page1.is_full(), "page 1 size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000E00), LogicalAddress::from(0x1009E00), 0x100);
    check!(compare_metadata(&page1.entry_at(METADATA_ARRAY_SIZE-1), &expected_md), "last entry of page 1 is different");
    check!(page2.size() == 2, "page 2 size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000F00), LogicalAddress::from(0x1009F00), 0x100);
    check!(compare_metadata(&page2.entry_at(0), &expected_md), "first entry of page 2 is different");
    let expected_md = new_md;
    check!(compare_metadata(&page2.entry_at(1), &expected_md), "second entry of page 2 is different");
    assume!(count_tracing_pages() == 2, "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A tracing table with two [`TracingPage`], of which the first is full and
///   the second contains multiple entries
///
/// ### When
///
/// - Requesting to insert an entry which address is in the range of addresses of the
///   second page
///
/// ### Then
///
/// - The entry is inserted successfully in the second page
/// - The first page is not modified
/// - The second page is updated correctly
/// - No new page is created
pub(super) fn insert_entry_second_page_multiple(page1:&mut TracingPage, page2:&mut TracingPage) {
    scenario!("Insert an entry (multi-page, second multiple)");
    // Given
    assume!(count_tracing_pages() == 2, "tracing pre-conditions");
    clear_page(page1);
    fill_page_taken(page1);
    assume!(page1.is_full(), "page1 pre-conditions");
    clear_page(page2);
    add_multiple_entries(page2);
    assume!(page2.size() == 4, "page2 pre-conditions");
    // When / Then
    let new_md = Metadata::new_taken(PhysicalAddress::from(0x1001000), LogicalAddress::from(0x100A000), 0x100);
    check!(insert::insert_entry(new_md, OWNER).is_ok(), "failed to insert");
    // Then
    check!(page1.is_full(), "page 1 size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000E00), LogicalAddress::from(0x1009E00), 0x100);
    check!(compare_metadata(&page1.entry_at(METADATA_ARRAY_SIZE-1), &expected_md), "last entry of page 1 is different");
    check!(page2.size() == 5, "page 2 size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000F00), LogicalAddress::from(0x1009F00), 0x100);
    check!(compare_metadata(&page2.entry_at(0), &expected_md), "entry 0 of page 2 is different");
    let expected_md = new_md;
    check!(compare_metadata(&page2.entry_at(1), &expected_md), "entry 1 of page 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1001100), LogicalAddress::from(0x100A100), 0x100);
    check!(compare_metadata(&page2.entry_at(2), &expected_md), "entry 2 of page 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1001200), LogicalAddress::from(0x100A200), 0x100);
    check!(compare_metadata(&page2.entry_at(3), &expected_md), "entry 3 of page 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1001400), LogicalAddress::from(0x100A400), 0x100);
    check!(compare_metadata(&page2.entry_at(4), &expected_md), "entry 4 of page 2 is different");
    assume!(count_tracing_pages() == 2, "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A tracing table with two [`TracingPage`], of which the first is full and
///   the second contains multiple entries
///
/// ### When
///
/// - Requesting to entirely remove an entry from the first page
///
/// ### Then
///
/// - The entry is removed successfully
/// - The pages are merged
/// - No page is created or deleted
pub(super) fn remove_entry_first_page(page1:&mut TracingPage, page2:&mut TracingPage) {
    scenario!("Remove an entry (multi-page, first entirely)");
    // Given
    assume!(count_tracing_pages() == 2, "tracing pre-conditions");
    clear_page(page1);
    fill_page_taken(page1);
    assume!(page1.is_full(), "page1 pre-conditions");
    clear_page(page2);
    add_multiple_entries(page2);
    assume!(page2.size() == 4, "page2 pre-conditions");
    // When / Then
    check!(remove::remove_space(0x1000.into(), 0x100, OWNER).is_ok(), "failed to remove");
    // Then
    check!(page1.is_full(), "page 1 size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1100), LogicalAddress::from(0xA100), 0x100);
    check!(compare_metadata(&page1.entry_at(0), &expected_md), "first entry of page 1 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000F00), LogicalAddress::from(0x1009F00), 0x100);
    check!(compare_metadata(&page1.entry_at(METADATA_ARRAY_SIZE-1), &expected_md), "last entry of page 1 is different");
    check!(page2.size() == 3, "page 2 size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1001100), LogicalAddress::from(0x100A100), 0x100);
    check!(compare_metadata(&page2.entry_at(0), &expected_md), "entry 0 of page 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1001200), LogicalAddress::from(0x100A200), 0x100);
    check!(compare_metadata(&page2.entry_at(1), &expected_md), "entry 1 of page 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1001400), LogicalAddress::from(0x100A400), 0x100);
    check!(compare_metadata(&page2.entry_at(2), &expected_md), "entry 2 of page 2 is different");
    assume!(count_tracing_pages() == 2, "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A tracing table with two [`TracingPage`], of which the first is full and
///   the second contains multiple entries
///
/// ### When
///
/// - Requesting to partially remove an entry (the middle part of it) from the first page
///
/// ### Then
///
/// - The entry is updated correctly
/// - Two new entries are created with the left-over size
/// - The first page is updated correctly
/// - The exceeding entry from the first page is moved to the second page
/// - The second page is updated correctly
/// - No new page is created
pub(super) fn remove_entry_partially_first_page(page1:&mut TracingPage, page2:&mut TracingPage) {
    scenario!("Remove an entry (multi-page, first partially)");
    // Given
    assume!(count_tracing_pages() == 2, "tracing pre-conditions");
    clear_page(page1);
    fill_page_alternate(page1);
    assume!(page1.is_full(), "page1 pre-conditions");
    clear_page(page2);
    add_multiple_entries(page2);
    assume!(page2.size() == 4, "page2 pre-conditions");
    // When / Then
    check!(remove::remove_space(0x1140.into(), 0x80, OWNER).is_ok(), "failed to remove");
    // Then
    check!(page1.is_full(), "page 1 size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000), LogicalAddress::from(0xA000), 0x100);
    check!(compare_metadata(&page1.entry_at(0), &expected_md), "entry 0 of page 1 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1100), LogicalAddress::from(0xA100), 0x40);
    check!(compare_metadata(&page1.entry_at(1), &expected_md), "entry 1 of page 1 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x11C0), LogicalAddress::from(0xA1C0), 0x40);
    check!(compare_metadata(&page1.entry_at(2), &expected_md), "entry 2 of page 1 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1000D00), LogicalAddress::from(0x1009D00), 0x100);
    check!(compare_metadata(&page1.entry_at(METADATA_ARRAY_SIZE-1), &expected_md), "last entry of page 1 is different");
    check!(page2.size() == 5, "page 2 size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000E00), LogicalAddress::from(0x1009E00), 0x100);
    check!(compare_metadata(&page2.entry_at(0), &expected_md), "entry 0 of page 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000F00), LogicalAddress::from(0x1009F00), 0x100);
    check!(compare_metadata(&page2.entry_at(1), &expected_md), "entry 1 of page 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1001100), LogicalAddress::from(0x100A100), 0x100);
    check!(compare_metadata(&page2.entry_at(2), &expected_md), "entry 2 of page 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1001200), LogicalAddress::from(0x100A200), 0x100);
    check!(compare_metadata(&page2.entry_at(3), &expected_md), "entry 3 of page 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1001400), LogicalAddress::from(0x100A400), 0x100);
    check!(compare_metadata(&page2.entry_at(4), &expected_md), "entry 4 of page 2 is different");
    assume!(count_tracing_pages() == 2, "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A tracing table with two [`TracingPage`], of which the first is full and
///   the second contains multiple entries
///
/// ### When
///
/// - Requesting to remove an amount of space that starts with the last entry of
///   the first page and ends with the first entry of the second page
///
/// ### Then
///
/// - The last entry of the first page is removed successfully
/// - The first page is updated correctly
/// - The first entry of the second page is removed successfully
/// - The second page is updated correctly
/// - The pages are merged
/// - No new page is created
pub(super) fn remove_entry_first_page_last_with_reminder(page1:&mut TracingPage, page2:&mut TracingPage) {
    scenario!("Remove an entry (multi-page, first reminder)");
    // Given
    assume!(count_tracing_pages() == 2, "tracing pre-conditions");
    clear_page(page1);
    fill_page_taken(page1);
    assume!(page1.is_full(), "page1 pre-conditions");
    clear_page(page2);
    add_multiple_entries(page2);
    assume!(page2.size() == 4, "page2 pre-conditions");
    // When / Then
    check!(remove::remove_space(0x1000E00.into(), 0x200, OWNER).is_ok(), "failed to remove");
    // Then
    check!(page1.is_full(), "page 1 size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000), LogicalAddress::from(0xA000), 0x100);
    check!(compare_metadata(&page1.entry_at(0), &expected_md), "first entry of page 1 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1001100), LogicalAddress::from(0x100A100), 0x100);
    check!(compare_metadata(&page1.entry_at(METADATA_ARRAY_SIZE-1), &expected_md), "last entry of page 1 is different");
    check!(page2.size() == 2, "page 2 size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1001200), LogicalAddress::from(0x100A200), 0x100);
    check!(compare_metadata(&page2.entry_at(0), &expected_md), "entry 0 of page 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1001400), LogicalAddress::from(0x100A400), 0x100);
    check!(compare_metadata(&page2.entry_at(1), &expected_md), "entry 1 of page 2 is different");
    assume!(count_tracing_pages() == 2, "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A tracing table with two [`TracingPage`], of which the first is full and
///   the second contains multiple entries
///
/// ### When
///
/// - Requesting to entirely remove an entry from the second page
///
/// ### Then
///
/// - The entry is removed successfully
/// - The first page is not modified
/// - The second page is updated correctly
/// - No page is created or deleted
pub(super) fn remove_entry_second_page(page1:&mut TracingPage, page2:&mut TracingPage) {
    scenario!("Remove an entry (multi-page, second entirely)");
    // Given
    assume!(count_tracing_pages() == 2, "tracing pre-conditions");
    clear_page(page1);
    fill_page_taken(page1);
    assume!(page1.is_full(), "page1 pre-conditions");
    clear_page(page2);
    add_multiple_entries(page2);
    assume!(page2.size() == 4, "page2 pre-conditions");
    // When / Then
    check!(remove::remove_space(0x1001100.into(), 0x100, OWNER).is_ok(), "failed to remove");
    // Then
    check!(page1.is_full(), "page 1 size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000E00), LogicalAddress::from(0x1009E00), 0x100);
    check!(compare_metadata(&page1.entry_at(METADATA_ARRAY_SIZE-1), &expected_md), "last entry of page 1 is different");
    check!(page2.size() == 3, "page 2 size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000F00), LogicalAddress::from(0x1009F00), 0x100);
    check!(compare_metadata(&page2.entry_at(0), &expected_md), "entry 0 of page 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1001200), LogicalAddress::from(0x100A200), 0x100);
    check!(compare_metadata(&page2.entry_at(1), &expected_md), "entry 1 of page 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1001400), LogicalAddress::from(0x100A400), 0x100);
    check!(compare_metadata(&page2.entry_at(2), &expected_md), "entry 2 of page 2 is different");
    assume!(count_tracing_pages() == 2, "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A tracing table with two [`TracingPage`], of which the first is full and
///   the second contains multiple entries
///
/// ### When
///
/// - Requesting to partially take a free entry (the middle part of it) of the
///   first page
///
/// ### Then
///
/// - The entry is updated correctly
/// - Two new entries are created with the left-over size
/// - The exceeding entries from the first page are moved to the second page
/// - No new page is created
pub(super) fn take_entry_partially_first_page(page1:&mut TracingPage, page2:&mut TracingPage) {
    scenario!("Take an entry (multi-page, first partially)");
    // Given
    assume!(count_tracing_pages() == 2, "tracing pre-conditions");
    clear_page(page1);
    fill_page_alternate(page1);
    assume!(page1.is_full(), "page1 pre-conditions");
    clear_page(page2);
    add_multiple_entries(page2);
    assume!(page2.size() == 4, "page2 pre-conditions");
    // When / Then
    check!(take::take_available_space(0x1140.into(), 0x80, OWNER).is_ok(), "failed to take");
    // Then
    check!(page1.is_full(), "page 1 size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000), LogicalAddress::from(0xA000), 0x100);
    check!(compare_metadata(&page1.entry_at(0), &expected_md), "entry 0 of page 1 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1100), LogicalAddress::from(0xA100), 0x40);
    check!(compare_metadata(&page1.entry_at(1), &expected_md), "entry 1 of page 1 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1140), LogicalAddress::from(0xA140), 0x80);
    check!(compare_metadata(&page1.entry_at(2), &expected_md), "entry 2 of page 1 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x11C0), LogicalAddress::from(0xA1C0), 0x40);
    check!(compare_metadata(&page1.entry_at(3), &expected_md), "entry 3 of page 1 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000C00), LogicalAddress::from(0x1009C00), 0x100);
    check!(compare_metadata(&page1.entry_at(METADATA_ARRAY_SIZE-1), &expected_md), "last entry of page 1 is different");
    check!(page2.size() == 6, "page 2 size is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1000D00), LogicalAddress::from(0x1009D00), 0x100);
    check!(compare_metadata(&page2.entry_at(0), &expected_md), "entry 0 of page 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000E00), LogicalAddress::from(0x1009E00), 0x100);
    check!(compare_metadata(&page2.entry_at(1), &expected_md), "entry 1 of page 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000F00), LogicalAddress::from(0x1009F00), 0x100);
    check!(compare_metadata(&page2.entry_at(2), &expected_md), "entry 2 of page 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1001100), LogicalAddress::from(0x100A100), 0x100);
    check!(compare_metadata(&page2.entry_at(3), &expected_md), "entry 3 of page 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1001200), LogicalAddress::from(0x100A200), 0x100);
    check!(compare_metadata(&page2.entry_at(4), &expected_md), "entry 4 of page 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1001400), LogicalAddress::from(0x100A400), 0x100);
    check!(compare_metadata(&page2.entry_at(5), &expected_md), "entry 5 of page 2 is different");
    assume!(count_tracing_pages() == 2, "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A tracing table with two [`TracingPage`], of which the first is full and
///   the second contains multiple entries
///
/// ### When
///
/// - Requesting to entirely take a free entry of the second page
///
/// ### Then
///
/// - The entry is updated successfully
/// - The first page is not modified
/// - The second page is updated correctly
/// - No page is created or deleted
pub(super) fn take_entry_entirely_second_page(page1:&mut TracingPage, page2:&mut TracingPage) {
    scenario!("Take an entry (multi-page, second entirely)");
    // Given
    assume!(count_tracing_pages() == 2, "tracing pre-conditions");
    clear_page(page1);
    fill_page_taken(page1);
    assume!(page1.is_full(), "page1 pre-conditions");
    clear_page(page2);
    add_multiple_entries(page2);
    page2.entry_at_mut(0).set_free();
    assume!(page2.size() == 4, "page2 pre-conditions");
    // When / Then
    check!(take::take_available_space(0x1000F00.into(), 0x100, OWNER).is_ok(), "failed to take");
    // Then
    check!(page1.is_full(), "page 1 size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000E00), LogicalAddress::from(0x1009E00), 0x100);
    check!(compare_metadata(&page1.entry_at(METADATA_ARRAY_SIZE-1), &expected_md), "last entry of page 1 is different");
    check!(page2.size() == 4, "page 2 size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000F00), LogicalAddress::from(0x1009F00), 0x100);
    check!(compare_metadata(&page2.entry_at(0), &expected_md), "entry 0 of page 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1001100), LogicalAddress::from(0x100A100), 0x100);
    check!(compare_metadata(&page2.entry_at(1), &expected_md), "entry 1 of page 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1001200), LogicalAddress::from(0x100A200), 0x100);
    check!(compare_metadata(&page2.entry_at(2), &expected_md), "entry 2 of page 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1001400), LogicalAddress::from(0x100A400), 0x100);
    check!(compare_metadata(&page2.entry_at(3), &expected_md), "entry 3 of page 2 is different");
    assume!(count_tracing_pages() == 2, "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A tracing table with two [`TracingPage`], of which the first is full and
///   the second contains multiple entries
///
/// ### When
///
/// - Requesting to drop a taken entry of the first page
///
/// ### Then
///
/// - The entry is updated correctly
/// - The second page is not modified
/// - No new page is created
pub(super) fn drop_entry_first_page(page1:&mut TracingPage, page2:&mut TracingPage) {
    scenario!("Drop an entry (multi-page, first)");
    // Given
    assume!(count_tracing_pages() == 2, "tracing pre-conditions");
    clear_page(page1);
    fill_page_taken(page1);
    assume!(page1.is_full(), "page1 pre-conditions");
    clear_page(page2);
    add_multiple_entries(page2);
    assume!(page2.size() == 4, "page2 pre-conditions");
    // When / Then
    check!(drop::drop_occupied_space(0x1000.into(), OWNER).is_ok(), "failed to drop");
    // Then
    check!(page1.is_full(), "page 1 size is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1000), LogicalAddress::from(0xA000), 0x100);
    check!(compare_metadata(&page1.entry_at(0), &expected_md), "first entry of page 1 is different");
    check!(page2.size() == 4, "page 2 size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000F00), LogicalAddress::from(0x1009F00), 0x100);
    check!(compare_metadata(&page2.entry_at(0), &expected_md), "entry 0 of page 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1001100), LogicalAddress::from(0x100A100), 0x100);
    check!(compare_metadata(&page2.entry_at(1), &expected_md), "entry 1 of page 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1001200), LogicalAddress::from(0x100A200), 0x100);
    check!(compare_metadata(&page2.entry_at(2), &expected_md), "entry 2 of page 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1001400), LogicalAddress::from(0x100A400), 0x100);
    check!(compare_metadata(&page2.entry_at(3), &expected_md), "entry 3 of page 2 is different");
    assume!(count_tracing_pages() == 2, "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A tracing table with two [`TracingPage`], of which the first is full and
///   the second contains multiple entries
///
/// ### When
///
/// - Requesting to drop a taken entry of the second page
///
/// ### Then
///
/// - The entry is updated successfully
/// - The first page is not modified
/// - The second page is updated correctly
/// - No page is created or deleted
pub(super) fn drop_entry_second_page(page1:&mut TracingPage, page2:&mut TracingPage) {
    scenario!("Drop an entry (multi-page, second)");
    // Given
    assume!(count_tracing_pages() == 2, "tracing pre-conditions");
    clear_page(page1);
    fill_page_taken(page1);
    assume!(page1.is_full(), "page1 pre-conditions");
    clear_page(page2);
    add_multiple_entries(page2);
    assume!(page2.size() == 4, "page2 pre-conditions");
    // When / Then
    check!(drop::drop_occupied_space(0x1001100.into(), OWNER).is_ok(), "failed to drop");
    // Then
    check!(page1.is_full(), "page 1 size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000), LogicalAddress::from(0xA000), 0x100);
    check!(compare_metadata(&page1.entry_at(0), &expected_md), "first entry of page 1 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000E00), LogicalAddress::from(0x1009E00), 0x100);
    check!(compare_metadata(&page1.entry_at(METADATA_ARRAY_SIZE-1), &expected_md), "last entry of page 1 is different");
    check!(page2.size() == 4, "page 2 size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000F00), LogicalAddress::from(0x1009F00), 0x100);
    check!(compare_metadata(&page2.entry_at(0), &expected_md), "entry 0 of page 2 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1001100), LogicalAddress::from(0x100A100), 0x100);
    check!(compare_metadata(&page2.entry_at(1), &expected_md), "entry 1 of page 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1001200), LogicalAddress::from(0x100A200), 0x100);
    check!(compare_metadata(&page2.entry_at(2), &expected_md), "entry 2 of page 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1001400), LogicalAddress::from(0x100A400), 0x100);
    check!(compare_metadata(&page2.entry_at(3), &expected_md), "entry 3 of page 2 is different");
    assume!(count_tracing_pages() == 2, "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A tracing table with two [`TracingPage`], of which the first is full and
///   the second contains multiple entries
///
/// ### When
///
/// - Requesting to resize a taken entry of the first page to a smaller size
///
/// ### Then
///
/// - The entry is updated correctly
/// - A new entry is created with the left-over space
/// - The first page is updated correctly
/// - The exceeding entry from the first page is moved correctly to the second page
/// - The second page is updated correctly
/// - No new page is created
pub(super) fn update_entry_resize_smaller_first_page(page1:&mut TracingPage, page2:&mut TracingPage) {
    scenario!("Resize an entry to a smaller size (multi-page, first)");
    // Given
    assume!(count_tracing_pages() == 2, "tracing pre-conditions");
    clear_page(page1);
    fill_page_taken(page1);
    assume!(page1.is_full(), "page1 pre-conditions");
    clear_page(page2);
    add_multiple_entries(page2);
    assume!(page2.size() == 4, "page2 pre-conditions");
    // When / Then
    check!(update::resize(0x1000.into(), 0x80, OWNER).is_ok(), "failed to resize");
    // Then
    check!(page1.is_full(), "page 1 size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000), LogicalAddress::from(0xA000), 0x80);
    check!(compare_metadata(&page1.entry_at(0), &expected_md), "entry 0 of page 1 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1080), LogicalAddress::from(0xA080), 0x80);
    check!(compare_metadata(&page1.entry_at(1), &expected_md), "entry 1 of page 1 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000D00), LogicalAddress::from(0x1009D00), 0x100);
    check!(compare_metadata(&page1.entry_at(METADATA_ARRAY_SIZE-1), &expected_md), "last entry of page 1 is different");
    check!(page2.size() == 5, "page 2 size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000E00), LogicalAddress::from(0x1009E00), 0x100);
    check!(compare_metadata(&page2.entry_at(0), &expected_md), "entry 0 of page 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000F00), LogicalAddress::from(0x1009F00), 0x100);
    check!(compare_metadata(&page2.entry_at(1), &expected_md), "entry 1 of page 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1001100), LogicalAddress::from(0x100A100), 0x100);
    check!(compare_metadata(&page2.entry_at(2), &expected_md), "entry 2 of page 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1001200), LogicalAddress::from(0x100A200), 0x100);
    check!(compare_metadata(&page2.entry_at(3), &expected_md), "entry 3 of page 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1001400), LogicalAddress::from(0x100A400), 0x100);
    check!(compare_metadata(&page2.entry_at(4), &expected_md), "entry 4 of page 2 is different");
    assume!(count_tracing_pages() == 2, "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A tracing table with two [`TracingPage`], of which the first is full and
///   the second contains multiple entries
///
/// ### When
///
/// - Requesting to resize a taken entry of the first page, which is contiguous to
///   a free entry, to a bigger size
///
/// ### Then
///
/// - The entry is updated correctly
/// - The contiguous free entry is completely drained
/// - The first page is updated correctly
/// - The pages are merged
/// - No new page is created
pub(super) fn update_entry_resize_bigger_first_page(page1:&mut TracingPage, page2:&mut TracingPage) {
    scenario!("Resize an entry to a bigger size (multi-page, first)");
    // Given
    assume!(count_tracing_pages() == 2, "tracing pre-conditions");
    clear_page(page1);
    fill_page_alternate(page1);
    assume!(page1.is_full(), "page1 pre-conditions");
    clear_page(page2);
    add_multiple_entries(page2);
    assume!(page2.size() == 4, "page2 pre-conditions");
    // When / Then
    check!(update::resize(0x1000.into(), 0x200, OWNER).is_ok(), "failed to resize");
    // Then
    check!(page1.is_full(), "page 1 size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000), LogicalAddress::from(0xA000), 0x200);
    check!(compare_metadata(&page1.entry_at(0), &expected_md), "entry 0 of page 1 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1200), LogicalAddress::from(0xA200), 0x100);
    check!(compare_metadata(&page1.entry_at(1), &expected_md), "entry 1 of page 1 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000F00), LogicalAddress::from(0x1009F00), 0x100);
    check!(compare_metadata(&page1.entry_at(METADATA_ARRAY_SIZE-1), &expected_md), "last entry of page 1 is different");
    check!(page2.size() == 3, "page 2 size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1001100), LogicalAddress::from(0x100A100), 0x100);
    check!(compare_metadata(&page2.entry_at(0), &expected_md), "entry 0 of page 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1001200), LogicalAddress::from(0x100A200), 0x100);
    check!(compare_metadata(&page2.entry_at(1), &expected_md), "entry 1 of page 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1001400), LogicalAddress::from(0x100A400), 0x100);
    check!(compare_metadata(&page2.entry_at(2), &expected_md), "entry 2 of page 2 is different");
    assume!(count_tracing_pages() == 2, "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A tracing table with two [`TracingPage`], of which the first is full and
///   the second contains multiple entries
///
/// ### When
///
/// - Requesting to resize a taken entry of the second page to a smaller size
///
/// ### Then
///
/// - The entry is updated correctly
/// - A new entry is created with the left-over space
/// - The first page is not modified
/// - The second page is updated correctly
/// - No new page is created
pub(super) fn update_entry_resize_smaller_second_page(page1:&mut TracingPage, page2:&mut TracingPage) {
    scenario!("Resize an entry to a smaller size (multi-page, second)");
    // Given
    assume!(count_tracing_pages() == 2, "tracing pre-conditions");
    clear_page(page1);
    fill_page_taken(page1);
    assume!(page1.is_full(), "page1 pre-conditions");
    clear_page(page2);
    add_multiple_entries(page2);
    assume!(page2.size() == 4, "page2 pre-conditions");
    // When / Then
    check!(update::resize(0x1001100.into(), 0x80, OWNER).is_ok(), "failed to resize");
    // Then
    check!(page1.is_full(), "page 1 size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000), LogicalAddress::from(0xA000), 0x100);
    check!(compare_metadata(&page1.entry_at(0), &expected_md), "first entry of page 1 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000E00), LogicalAddress::from(0x1009E00), 0x100);
    check!(compare_metadata(&page1.entry_at(METADATA_ARRAY_SIZE-1), &expected_md), "last entry of page 1 is different");
    check!(page2.size() == 5, "page 2 size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000F00), LogicalAddress::from(0x1009F00), 0x100);
    check!(compare_metadata(&page2.entry_at(0), &expected_md), "entry 0 of page 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1001100), LogicalAddress::from(0x100A100), 0x80);
    check!(compare_metadata(&page2.entry_at(1), &expected_md), "entry 1 of page 2 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1001180), LogicalAddress::from(0x100A180), 0x80);
    check!(compare_metadata(&page2.entry_at(2), &expected_md), "entry 2 of page 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1001200), LogicalAddress::from(0x100A200), 0x100);
    check!(compare_metadata(&page2.entry_at(3), &expected_md), "entry 3 of page 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1001400), LogicalAddress::from(0x100A400), 0x100);
    check!(compare_metadata(&page2.entry_at(4), &expected_md), "entry 4 of page 2 is different");
    assume!(count_tracing_pages() == 2, "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A tracing table with two [`TracingPage`], of which the first is full and
///   the second contains multiple entries
///
/// ### When
///
/// - Requesting to check whether a taken entry of the first page can be resized
///   to a smaller size
///
/// ### Then
///
/// - The request is processed successfully
/// - The pages are not modified
/// - No new page is created
pub(super) fn query_relocation_inplace_first_page(page1:&mut TracingPage, page2:&mut TracingPage) {
    scenario!("Query relocation inplace to a smaller size (multi-page, first)");
    // Given
    assume!(count_tracing_pages() == 2, "tracing pre-conditions");
    clear_page(page1);
    fill_page_taken(page1);
    assume!(page1.is_full(), "page1 pre-conditions");
    clear_page(page2);
    add_multiple_entries(page2);
    assume!(page2.size() == 4, "page2 pre-conditions");
    // When
    let query_result = query::can_relocate_inplace(0xA000.into(), 0x80, OWNER);
    // Then
    check!(query_result.is_ok(), "failed to query");
    check!(query_result.get_or_panic().0 == true, "query result is different");
    check!(page1.is_full(), "page 1 size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000), LogicalAddress::from(0xA000), 0x100);
    check!(compare_metadata(&page1.entry_at(0), &expected_md), "first entry of page 1 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000E00), LogicalAddress::from(0x1009E00), 0x100);
    check!(compare_metadata(&page1.entry_at(METADATA_ARRAY_SIZE-1), &expected_md), "last entry of page 1 is different");
    check!(page2.size() == 4, "page 2 size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000F00), LogicalAddress::from(0x1009F00), 0x100);
    check!(compare_metadata(&page2.entry_at(0), &expected_md), "entry 0 of page 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1001100), LogicalAddress::from(0x100A100), 0x100);
    check!(compare_metadata(&page2.entry_at(1), &expected_md), "entry 1 of page 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1001200), LogicalAddress::from(0x100A200), 0x100);
    check!(compare_metadata(&page2.entry_at(2), &expected_md), "entry 2 of page 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1001400), LogicalAddress::from(0x100A400), 0x100);
    check!(compare_metadata(&page2.entry_at(3), &expected_md), "entry 3 of page 2 is different");
    assume!(count_tracing_pages() == 2, "tracing post-conditions");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A tracing table with two [`TracingPage`], of which the first is full and
///   the second contains multiple entries
///
/// ### When
///
/// - Requesting to check whether a taken entry of the second page can be resized
///   to a smaller size
///
/// ### Then
///
/// - The request is processed successfully
/// - The pages are not modified
/// - No new page is created
pub(super) fn query_relocation_inplace_second_page(page1:&mut TracingPage, page2:&mut TracingPage) {
    scenario!("Query relocation inplace to a smaller size (multi-page, second)");
    // Given
    assume!(count_tracing_pages() == 2, "tracing pre-conditions");
    clear_page(page1);
    fill_page_taken(page1);
    assume!(page1.is_full(), "page1 pre-conditions");
    clear_page(page2);
    add_multiple_entries(page2);
    assume!(page2.size() == 4, "page2 pre-conditions");
    // When
    let query_result = query::can_relocate_inplace(0x1009F00.into(), 0x80, OWNER);
    // Then
    check!(query_result.is_ok(), "failed to query");
    check!(query_result.get_or_panic().0 == true, "query result is different");
    check!(page1.is_full(), "page 1 size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000), LogicalAddress::from(0xA000), 0x100);
    check!(compare_metadata(&page1.entry_at(0), &expected_md), "first entry of page 1 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000E00), LogicalAddress::from(0x1009E00), 0x100);
    check!(compare_metadata(&page1.entry_at(METADATA_ARRAY_SIZE-1), &expected_md), "last entry of page 1 is different");
    check!(page2.size() == 4, "page 2 size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000F00), LogicalAddress::from(0x1009F00), 0x100);
    check!(compare_metadata(&page2.entry_at(0), &expected_md), "entry 0 of page 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1001100), LogicalAddress::from(0x100A100), 0x100);
    check!(compare_metadata(&page2.entry_at(1), &expected_md), "entry 1 of page 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1001200), LogicalAddress::from(0x100A200), 0x100);
    check!(compare_metadata(&page2.entry_at(2), &expected_md), "entry 2 of page 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1001400), LogicalAddress::from(0x100A400), 0x100);
    check!(compare_metadata(&page2.entry_at(3), &expected_md), "entry 3 of page 2 is different");
    assume!(count_tracing_pages() == 2, "tracing post-conditions");
    test_passed!();
    wait!();
}

}

fn add_second_page() {
    let _ = create::create_tracing_page(test::OWNER).get_or_panic();
}

fn remove_second_page() {
    use crate::traits::ok_or_panic::OkOrPanic;
    delete::delete_tracing_page(get_page_paddr(1), test::OWNER).ok_or_panic();
}

fn get_page<'a>(n:usize) -> &'a mut TracingPage {
    unsafe { &mut *TracingPage::from_table_entry(TracingPagesIterator::new(test::OWNER).nth(n).unwrap()) }
}

fn get_page_paddr(n:usize) -> PhysicalAddress {
    PhysicalAddress::from(TracingPage::from_table_entry(TracingPagesIterator::new(test::OWNER).nth(n).unwrap()) as u64)
}

fn clear_page(page:&mut TracingPage) {
    page.clear();
}

// to be used on the first page during single-page tests
fn add_entries(page:&mut TracingPage) {
    let _ = page.append_unchecked(Metadata::new_taken(PhysicalAddress::from(0x1000), LogicalAddress::from(0xA000), 0x100));
    let _ = page.append_unchecked(Metadata::new_free(PhysicalAddress::from(0x2000), LogicalAddress::from(0xB000), 0x100));
    let _ = page.append_unchecked(Metadata::new_free(PhysicalAddress::from(0x3000), LogicalAddress::from(0xC000), 0x100));
    let _ = page.append_unchecked(Metadata::new_taken(PhysicalAddress::from(0x4000), LogicalAddress::from(0xD000), 0x100));
}

// to be used on the first page during multi-page tests
fn fill_page_alternate(page:&mut TracingPage) {
    // first -> status: taken, paddr: 0x1000, laddr: 0xA000, size: 0x100
    // second -> status: free, paddr: 0x1100, laddr: 0xA100, size: 0x100
    // ...
    // second-last -> status: free, paddr: 0x1000D00, laddr: 0x1009D00, size: 0x100
    // last -> status: taken, paddr: 0x1000E00, laddr: 0x1009E00, size: 0x100
    page.fill_alternate(PhysicalAddress::from(0x1000), LogicalAddress::from(0xA000), 0x100);
}
fn fill_page_taken(page:&mut TracingPage) {
    // first -> status: taken, paddr: 0x1000, laddr: 0xA000, size: 0x100
    // second -> status: taken, paddr: 0x1100, laddr: 0xA100, size: 0x100
    // ...
    // second-last -> status: taken, paddr: 0x1000D00, laddr: 0x1009D00, size: 0x100
    // last -> status: taken, paddr: 0x1000E00, laddr: 0x1009E00, size: 0x100
    page.fill_taken(PhysicalAddress::from(0x1000), LogicalAddress::from(0xA000), 0x100);
}

// to be used on the second page during multi-page tests
fn add_taken_entry(page:&mut TracingPage) {
    let _ = page.append_unchecked(Metadata::new_taken(PhysicalAddress::from(0x2000000), LogicalAddress::from(0xB000000), 0x100));
}
fn add_free_entry(page:&mut TracingPage) {
    let _ = page.append_unchecked(Metadata::new_free(PhysicalAddress::from(0x2000000), LogicalAddress::from(0xB000000), 0x100));
}

// to be used on the second page during multi-page tests
fn add_contiguous_taken_entry(page:&mut TracingPage) {
    let _ = page.append_unchecked(Metadata::new_taken(PhysicalAddress::from(0x1000F00), LogicalAddress::from(0x1009F00), 0x100));
}
fn add_contiguous_free_entry(page:&mut TracingPage) {
    let _ = page.append_unchecked(Metadata::new_free(PhysicalAddress::from(0x1000F00), LogicalAddress::from(0x1009F00), 0x100));
}

// to be used on the second page during multi-page tests
fn add_multiple_entries(page:&mut TracingPage) {
    // contiguous
    let _ = page.append_unchecked(Metadata::new_taken(PhysicalAddress::from(0x1000F00), LogicalAddress::from(0x1009F00), 0x100));
    // non-contiguous (missing -> paddr: 0x1001000, laddr: 0x100A000, size: 0x100)
    let _ = page.append_unchecked(Metadata::new_taken(PhysicalAddress::from(0x1001100), LogicalAddress::from(0x100A100), 0x100));
    // contiguous
    let _ = page.append_unchecked(Metadata::new_taken(PhysicalAddress::from(0x1001200), LogicalAddress::from(0x100A200), 0x100));
    // non-contiguous (missing -> paddr: 0x1001300, laddr: 0x100A300, size: 0x100)
    let _ = page.append_unchecked(Metadata::new_taken(PhysicalAddress::from(0x1001400), LogicalAddress::from(0x100A400), 0x100));
}
