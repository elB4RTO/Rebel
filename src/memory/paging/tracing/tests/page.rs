use crate::test::*;
use crate::memory::paging::tracing::tests::*;

use crate::GetOrPanic;
use crate::memory::{SIZE_2MiB, MemoryOwner, LogicalAddress, PhysicalAddress};
use crate::memory::paging::tracing::*;

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
/// - The tracing table of the process is clean
pub(crate)
fn run_all_tests() {
    module!("memory::paging::tracing", "page");
    let page = get_page();

    // ensure correct page size
    test::page_size();

    // empty page state
    test::empty_page_checks(page);
    test::empty_page_contains_addresses(page);
    // push on empty page
    test::push_on_empty_page(page);
    // append on empty page
    test::append_on_empty_page(page);
    // insert on empty page
    test::insert_on_empty_page(page);
    // pop on empty page
    test::fail__pop_on_empty_page(page);

    // page state
    test::contains_addresses(page);
    // push on page with entries
    test::fail__push_entry_higher(page);
    test::push_taken_entry(page);
    test::push_free_entry_contiguous(page);
    test::push_free_entry_non_contiguous(page);
    // append an entry
    test::fail__append_entry_lower(page);
    test::append_taken_entry(page);
    test::append_free_entry_contiguous(page);
    test::append_free_entry_non_contiguous(page);
    // insert an entry
    test::insert_entry_lower(page);
    test::insert_entry_higher(page);
    test::insert_taken_entry(page);
    test::insert_free_entry_between_taken(page);
    test::insert_free_entry_before_free_contiguous(page);
    test::insert_free_entry_before_free_non_contiguous(page);
    test::insert_free_entry_after_free_contiguous(page);
    test::insert_free_entry_after_free_non_contiguous(page);
    // pop an entry
    test::pop_entry(page);
    // extract an entry
    test::fail__extract_entry_out_of_bounds(page);
    test::extract_entry_first(page);
    test::extract_entry_middle(page);
    test::extract_entry_last(page);
    // remove allocations space
    test::remove_free_entry_partially_lower(page);
    test::remove_free_entry_partially_higher(page);
    test::remove_free_entry_partially_middle(page);
    test::remove_free_entry_entirely(page);
    test::fail__remove_taken_entry_partially(page);
    test::remove_taken_entry_entirely(page);
    test::fail__remove_non_contiguous_entries_at_once(page);
    test::remove_contiguous_entries_at_once(page);
    // take free space
    test::fail__take_taken_entry_entirely(page);
    test::take_free_entry_entirely(page);
    test::fail__take_taken_entry_partially(page);
    test::take_free_entry_partially_lower(page);
    test::take_free_entry_partially_higher(page);
    test::take_free_entry_partially_middle(page);
    test::fail__take_free_entry_smaller(page);
    // drop taken space
    test::fail__drop_free_entry(page);
    test::drop_taken_entry_noncontiguous(page);
    test::drop_taken_entry_contiguous_right(page);
    test::drop_taken_entry_contiguous_left(page);
    test::drop_taken_entry_contiguous_both(page);
    // resize taken space
    test::fail__resize_free_entry_smaller(page);
    test::fail__resize_free_entry_bigger(page);
    test::resize_taken_entry_smaller_noncontiguous(page);
    test::fail__resize_taken_entry_bigger_noncontiguous(page);
    test::resize_taken_entry_smaller_contiguous_taken(page);
    test::fail__resize_taken_entry_bigger_contiguous_taken(page);
    test::resize_taken_entry_smaller_contiguous_free(page);
    test::resize_taken_entry_bigger_contiguous_free_equal(page);
    test::resize_taken_entry_bigger_contiguous_free_big(page);
    test::fail__resize_taken_entry_bigger_contiguous_free_small(page);
    test::fail__resize_taken_entry_bigger_last(page);

    // full page state
    test::full_page_checks(page);
    // push on full page
    test::push_on_full_page(page);
    // append on full page
    test::append_on_full_page(page);
    // insert on full page
    test::insert_on_full_page(page);
    // pop on full page
    test::pop_on_full_page(page);
    // remove on full page
    test::remove_on_full_page(page);
    test::remove_free_entry_partially_middle_on_full_page(page);
    test::remove_entries_from_last_with_reminder_on_full_page(page);
    test::remove_all_entries_on_full_page(page);
    // take on full page
    test::take_free_entry_partially_half_on_full_page(page);
    test::take_free_entry_partially_middle_on_full_page(page);
    // drop on full page
    test::drop_taken_entry_contiguous_both_on_full_page(page);
    // resize on full page
    test::resize_taken_entry_bigger_last_on_full_page(page);

    // clean-up
    clear_page(page);
}

mod test {

use super::*;

/// # Test
///
/// ### Given
///
/// - The definition of [`TracingPage`]
///
/// ### When
///
/// - Getting the size of a [`TracingPage`]
///
/// ### Then
///
/// - The size is 2 MiB
pub(super) fn page_size() {
    scenario!("Size of a page");
    // Given / When / Then
    let max_size = SIZE_2MiB;
    let min_size = SIZE_2MiB - core::mem::size_of::<Metadata>() as u64;
    let page_size = core::mem::size_of::<TracingPage>() as u64;
    test!(page_size < max_size && page_size > min_size, "TracingPage size has a wrong size");
    wait!();
}

/// # Test
///
/// ### Given
///
/// - An empty [`TracingPage`]
///
/// ### When
///
/// - Calling [`TracingPage::is_empty`]
/// - Calling [`TracingPage::is_full`]
/// - Iterating its entry throught a [`MetadataIterator`]
///
/// ### Then
///
/// - [`TracingPage::is_empty`] returns `true`
/// - [`TracingPage::is_full`] returns `false`
/// - The iterator suddenly returns [`None`] when calling [`Iterator::next`]
pub(super) fn empty_page_checks(page:&mut TracingPage) {
    scenario!("Checks on an empty page");
    // Given
    clear_page(page);
    // When / Then
    check!(page.is_empty() == true, "is_empty() returned false");
    // When / Then
    check!(page.is_full() == false, "is_full() returned true");
    // When / Then
    let mut md_iter = page.iterate();
    check!(md_iter.next() == None, "next() returned Some");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - An empty [`TracingPage`]
///
/// ### When
///
/// - Calling [`TracingPage::contains_paddr`] with an address
/// - Calling [`TracingPage::contains_paddr`] with a null address
/// - Calling [`TracingPage::contains_paddr_strict`] with an address
/// - Calling [`TracingPage::contains_paddr_strict`] with a null address
///
/// ### Then
///
/// - All the requests return `false`
pub(super) fn empty_page_contains_addresses(page:&mut TracingPage) {
    scenario!("Contains addresses");
    // Given
    assume!(page.is_empty() == true, "page pre-conditions");
    // When / Then
    check!(page.contains_paddr(0x100000.into()) == false, "page contains a paddr");
    // When / Then
    check!(page.contains_paddr(0x0.into()) == false, "page contains a null paddr");
    // When / Then
    check!(page.contains_paddr_strict(0x1FA00000.into()) == false, "page strictly contains a paddr");
    // When / Then
    check!(page.contains_paddr_strict(0x0.into()) == false, "page strictly contains a null paddr");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - An empty [`TracingPage`]
///
/// ### When
///
/// - Requesting to push a new entry
///
/// ### Then
///
/// - The entry is succesfully pushed
pub(super) fn push_on_empty_page(page:&mut TracingPage) {
    scenario!("Try push on empty page");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    let new_md = Metadata::new_taken(PhysicalAddress::from(0x4000), LogicalAddress::from(0xA0000), 0x1000);
    // When
    let push_result = page.try_push(new_md);
    // Then
    check!(push_result.is_ok(), "result is Err");
    check!(page.size() == 1, "page size is different");
    check!(compare_metadata(page.entry_at(0), &new_md), "entry is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] with some entries
///
/// ### When
///
/// - Calling [`TracingPage::contains_paddr`] with an address out-of-bounds
/// - Calling [`TracingPage::contains_paddr`] with an address inside bounds
/// - Calling [`TracingPage::contains_paddr`] with an address one-off the higher bound
/// - Calling [`TracingPage::contains_paddr_strict`] with an address out-of-bounds
/// - Calling [`TracingPage::contains_paddr_strict`] with an address inside bounds
/// - Calling [`TracingPage::contains_paddr_strict`] with an address one-off the higher bound
/// - Calling [`TracingPage::contains_paddr_strict`] with an address equal to the higher bound
///
/// ### Then
///
/// - [`TracingPage::contains_paddr`] returns `false`
/// - [`TracingPage::contains_paddr`] returns `true`
/// - [`TracingPage::contains_paddr`] returns `true`
/// - [`TracingPage::contains_paddr_strict`] returns `false`
/// - [`TracingPage::contains_paddr_strict`] returns `true`
pub(super) fn contains_addresses(page:&mut TracingPage) {
    scenario!("Contains addresses");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    let mut paddr = PhysicalAddress::from(0x1FA00000);
    let mut laddr = PhysicalAddress::from(0xFFFF400000200000);
    // When / Then
    check!(page.contains_paddr(0x100000.into()) == false, "page contains an out-of-bounds paddr");
    // When / Then
    check!(page.contains_paddr(0x10C00000.into()) == true, "page does not contain an inside-bounds paddr");
    // When / Then
    check!(page.contains_paddr(0x1FA00000.into()) == true, "page does not contain a paddr one-off the higher-bound");
    // When / Then
    check!(page.contains_paddr_strict(0x100000.into()) == false, "page strictly contains an out-of-bounds paddr");
    // When / Then
    check!(page.contains_paddr_strict(0x10C00000.into()) == true, "page does not strictly contain an inside-bounds paddr");
    // When / Then
    check!(page.contains_paddr_strict(0x1FA00000.into()) == false, "page strictly contains a paddr one-off the higher-bound");
    // When / Then
    check!(page.contains_paddr_strict(0x1F9FFFFF.into()) == true, "page does not strictly contain a paddr equal to the higher-bound");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - An empty [`TracingPage`]
///
/// ### When
///
/// - Requesting to append a new entry
///
/// ### Then
///
/// - The entry is succesfully appended
pub(super) fn append_on_empty_page(page:&mut TracingPage) {
    scenario!("Try append on empty page");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    let new_md = Metadata::new_taken(PhysicalAddress::from(0x4000), LogicalAddress::from(0xA0000), 0x1000);
    // When
    let append_result = page.try_append(new_md);
    // Then
    check!(append_result.is_ok(), "result is Err");
    check!(page.size() == 1, "page size is different");
    check!(compare_metadata(page.entry_at(0), &new_md), "entry is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - An empty [`TracingPage`]
/// - A new [`Metadata`] entry
///
/// ### When
///
/// - Requesting to insert the new entry
///
/// ### Then
///
/// - The entry is succesfully inserted
pub(super) fn insert_on_empty_page(page:&mut TracingPage) {
    scenario!("Insert on empty page");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    let new_md = Metadata::new_taken(PhysicalAddress::from(0x4000), LogicalAddress::from(0xA0000), 0x1000);
    // When
    let insert_result = page.insert(new_md);
    // Then
    check!(insert_result.is_ok(), "result is Err");
    check!(page.size() == 1, "page size is different");
    check!(compare_metadata(page.entry_at(0), &new_md), "entry is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - An empty [`TracingPage`]
///
/// ### When
///
/// - Requesting to pop an entry
///
/// ### Then
///
/// - No entry is popped
pub(super) fn fail__pop_on_empty_page(page:&mut TracingPage) {
    scenario!("Try pop on empty page");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    // When
    let pop_result = page.try_pop();
    // Then
    check!(pop_result.is_err(), "result is Ok");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to push an entry which [`PhysicalAddress`] is higher than
///   the first page entry
///
/// ### Then
///
/// - The request is refused regardless of whether the entry is taken or free
/// - The page is not modified
pub(super) fn fail__push_entry_higher(page:&mut TracingPage) {
    fail_scenario!("Try push an entry with higher address");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    let mut new_md = Metadata::new_taken(PhysicalAddress::from(0x10001000), LogicalAddress::from(0xA0000), 0x1000);
    // When / Then
    check!(page.try_push(new_md).is_err(), "result is Ok");
    check!(page.size() == 4, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "first entry is different");
    // When / Then
    new_md.set_free();
    check!(page.try_push(new_md).is_err(), "result is Ok");
    check!(page.size() == 4, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "first entry is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to push a taken entry which [`PhysicalAddress`] is lower than
///   the first page entry
///
/// ### Then
///
/// - The entry is pushed succesfully
/// - The returned excess is empty
/// - The page is updated correctly
pub(super) fn push_taken_entry(page:&mut TracingPage) {
    scenario!("Try push a taken entry");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    let new_md = Metadata::new_taken(PhysicalAddress::from(0x4000), LogicalAddress::from(0xA0000), 0x1000);
    // When
    let push_result = page.try_push(new_md);
    // Then
    check!(push_result.is_ok(), "failed to push");
    check!(push_result.get_or_panic().is_none(), "returned excess is not empty");
    check!(page.size() == 5, "page size is different");
    check!(compare_metadata(page.entry_at(0), &new_md), "entry 0 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF100000000000), 0x200000);
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x11000000), LogicalAddress::from(0xFFFF100000200000), 0x200000);
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1F800000), LogicalAddress::from(0xFFFF400000000000), 0x200000);
    check!(compare_metadata(page.entry_at(4), &expected_md), "entry 4 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries, of which the first is free
///
/// ### When
///
/// - Requesting to push a free entry which is contiguous to the first page entry
///
/// ### Then
///
/// - The entry is merged with the first page entry
/// - The returned excess is empty
pub(super) fn push_free_entry_contiguous(page:&mut TracingPage) {
    scenario!("Try push a free entry (contiguous)");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    *page.entry_at_mut(0) = Metadata::new_free(PhysicalAddress::from(0x10100000), LogicalAddress::from(0xFFFF000000100000), 0x100000);
    let new_md = Metadata::new_free(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x100000);
    // When
    let push_result = page.try_push(new_md);
    // Then
    check!(push_result.is_ok(), "failed to push");
    check!(push_result.get_or_panic().is_none(), "returned excess is not empty");
    check!(page.size() == 4, "page size is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF100000000000), 0x200000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x11000000), LogicalAddress::from(0xFFFF100000200000), 0x200000);
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1F800000), LogicalAddress::from(0xFFFF400000000000), 0x200000);
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries, of which the first is free
///
/// ### When
///
/// - Requesting to push a free entry which [`PhysicalAddress`] is lower than
///   the first page entry and not contiguous to it
///
/// ### Then
///
/// - The entry is pushed succesfully
/// - The returned excess is empty
/// - The page is updated correctly
pub(super) fn push_free_entry_non_contiguous(page:&mut TracingPage) {
    scenario!("Try push a free entry (non-contiguous)");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    page.entry_at_mut(0).set_free();
    let new_md = Metadata::new_free(PhysicalAddress::from(0xF00000), LogicalAddress::from(0xFFFFA00000000000), 0x100000);
    // When
    let push_result = page.try_push(new_md);
    // Then
    check!(push_result.is_ok(), "failed to push");
    check!(push_result.get_or_panic().is_none(), "returned excess is not empty");
    check!(page.size() == 5, "page size is different");
    check!(compare_metadata(page.entry_at(0), &new_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF100000000000), 0x200000);
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x11000000), LogicalAddress::from(0xFFFF100000200000), 0x200000);
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1F800000), LogicalAddress::from(0xFFFF400000000000), 0x200000);
    check!(compare_metadata(page.entry_at(4), &expected_md), "entry 4 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to append an entry which [`PhysicalAddress`] is lower than
///   the last page entry
///
/// ### Then
///
/// - The request is refused regardless of whether the entry is taken or free
/// - The page is not modified
pub(super) fn fail__append_entry_lower(page:&mut TracingPage) {
    fail_scenario!("Try append an entry with lower address");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    let mut new_md = Metadata::new_taken(PhysicalAddress::from(0x1F600000), LogicalAddress::from(0xFFFF400000200000), 0x100000);
    // When / Then
    check!(page.try_append(new_md).is_err(), "result is Ok");
    check!(page.size() == 4, "page size is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1F800000), LogicalAddress::from(0xFFFF400000000000), 0x200000);
    check!(compare_metadata(page.entry_at(3), &expected_md), "last entry is different");
    // When / Then
    new_md.set_free();
    check!(page.try_push(new_md).is_err(), "result is Ok");
    check!(page.size() == 4, "page size is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1F800000), LogicalAddress::from(0xFFFF400000000000), 0x200000);
    check!(compare_metadata(page.entry_at(3), &expected_md), "last entry is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to append a taken entry which [`PhysicalAddress`] is higher than
///   the last page entry
///
/// ### Then
///
/// - The higher entry is appended succesfully
/// - The returned excess is empty
/// - The page is updated correctly
pub(super) fn append_taken_entry(page:&mut TracingPage) {
    scenario!("Try append a taken entry");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    let mut new_md = Metadata::new_taken(PhysicalAddress::from(0x1FA00000), LogicalAddress::from(0xFFFF400000200000), 0x100000);
    // When
    let append_result = page.try_append(new_md);
    // Then
    check!(append_result.is_ok(), "failed to append");
    check!(append_result.get_or_panic().is_none(), "returned excess is not empty");
    check!(page.size() == 5, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF100000000000), 0x200000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x11000000), LogicalAddress::from(0xFFFF100000200000), 0x200000);
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1F800000), LogicalAddress::from(0xFFFF400000000000), 0x200000);
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    check!(compare_metadata(page.entry_at(4), &new_md), "entry 4 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries, of which the last is free
///
/// ### When
///
/// - Requesting to append a free entry which is contiguous to the last page entry
///
/// ### Then
///
/// - The entry is merged with the last page entry
/// - The returned excess is empty
pub(super) fn append_free_entry_contiguous(page:&mut TracingPage) {
    scenario!("Try append a free entry (contiguous)");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    let new_md = Metadata::new_free(PhysicalAddress::from(0x1FA00000), LogicalAddress::from(0xFFFF400000200000), 0x100000);
    // When
    let append_result = page.try_append(new_md);
    // Then
    check!(append_result.is_ok(), "failed to append");
    check!(append_result.get_or_panic().is_none(), "returned excess is not empty");
    check!(page.size() == 4, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF100000000000), 0x200000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x11000000), LogicalAddress::from(0xFFFF100000200000), 0x200000);
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1F800000), LogicalAddress::from(0xFFFF400000000000), 0x300000);
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries, of which the last is free
///
/// ### When
///
/// - Requesting to append a free entry which [`PhysicalAddress`] is higher than
///   the last page entry and not contiguous to it
///
/// ### Then
///
/// - The entry is appended succesfully
/// - The returned excess is empty
/// - The page is updated correctly
pub(super) fn append_free_entry_non_contiguous(page:&mut TracingPage) {
    scenario!("Try append a free entry (non-contiguous)");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    let new_md = Metadata::new_taken(PhysicalAddress::from(0x1FA00000), LogicalAddress::from(0xFFFF400000300000), 0x100000);
    // When
    let append_result = page.try_append(new_md);
    // Then
    check!(append_result.is_ok(), "failed to append");
    check!(append_result.get_or_panic().is_none(), "returned excess is not empty");
    check!(page.size() == 5, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF100000000000), 0x200000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x11000000), LogicalAddress::from(0xFFFF100000200000), 0x200000);
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1F800000), LogicalAddress::from(0xFFFF400000000000), 0x200000);
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    check!(compare_metadata(page.entry_at(4), &new_md), "entry 4 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to insert an entry which [`PhysicalAddress`] is lower than
///   the first page entry
///
/// ### Then
///
/// - The entry is inserted as first entry
/// - The returned excess is empty
/// - The page is updated correctly
pub(super) fn insert_entry_lower(page:&mut TracingPage) {
    scenario!("Insert an entry with lower address");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    let new_md = Metadata::new_taken(PhysicalAddress::from(0x4000), LogicalAddress::from(0xA0000), 0x1000);
    // When
    let insert_result = page.insert(new_md);
    // Then
    check!(insert_result.is_ok(), "failed to insert");
    check!(insert_result.get_or_panic().is_none(), "returned excess is not empty");
    check!(page.size() == 5, "page size is different");
    check!(compare_metadata(page.entry_at(0), &new_md), "entry 0 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF100000000000), 0x200000);
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x11000000), LogicalAddress::from(0xFFFF100000200000), 0x200000);
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1F800000), LogicalAddress::from(0xFFFF400000000000), 0x200000);
    check!(compare_metadata(page.entry_at(4), &expected_md), "entry 4 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to insert an entry which [`PhysicalAddress`] is higher then
///   the last page entry
///
/// ### Then
///
/// - The entry is inserted as last entry
/// - The returned excess is empty
/// - The page is updated correctly
pub(super) fn insert_entry_higher(page:&mut TracingPage) {
    scenario!("Insert an entry with higher address");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    let mut new_md = Metadata::new_taken(PhysicalAddress::from(0x1FA00000), LogicalAddress::from(0xFFFF400000200000), 0x100000);
    // When
    let insert_result = page.insert(new_md);
    // Then
    check!(insert_result.is_ok(), "failed to insert");
    check!(insert_result.get_or_panic().is_none(), "returned excess is not empty");
    check!(page.size() == 5, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF100000000000), 0x200000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x11000000), LogicalAddress::from(0xFFFF100000200000), 0x200000);
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1F800000), LogicalAddress::from(0xFFFF400000000000), 0x200000);
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    check!(compare_metadata(page.entry_at(4), &new_md), "entry 4 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to insert a taken entry which [`PhysicalAddress`] is higher then
///   the first page entry and lower than the last page entry
///
/// ### Then
///
/// - The entry is inserted successfully
/// - The returned excess is empty
/// - The page is updated correctly
pub(super) fn insert_taken_entry(page:&mut TracingPage) {
    scenario!("Insert a taken entry");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    let mut new_md = Metadata::new_taken(PhysicalAddress::from(0x10A00000), LogicalAddress::from(0xFFFF000000A00000), 0x100000);
    // When
    let insert_result = page.insert(new_md);
    // Then
    check!(insert_result.is_ok(), "failed to insert");
    check!(insert_result.get_or_panic().is_none(), "returned excess is not empty");
    check!(page.size() == 5, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF100000000000), 0x200000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    check!(compare_metadata(page.entry_at(2), &new_md), "entry 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x11000000), LogicalAddress::from(0xFFFF100000200000), 0x200000);
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1F800000), LogicalAddress::from(0xFFFF400000000000), 0x200000);
    check!(compare_metadata(page.entry_at(4), &expected_md), "entry 4 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to insert a free entry between two other taken entries
///
/// ### Then
///
/// - The entry is inserted successfully
/// - The returned excess is empty
/// - The page is updated correctly
pub(super) fn insert_free_entry_between_taken(page:&mut TracingPage) {
    scenario!("Insert a free entry between taken entries");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    page.entry_at_mut(1).set_taken();
    let mut new_md = Metadata::new_free(PhysicalAddress::from(0x10A00000), LogicalAddress::from(0xFFFFA00000000000), 0x100000);
    // When
    let insert_result = page.insert(new_md);
    // Then
    check!(insert_result.is_ok(), "failed to insert");
    check!(insert_result.get_or_panic().is_none(), "returned excess is not empty");
    check!(page.size() == 5, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF100000000000), 0x200000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    check!(compare_metadata(page.entry_at(2), &new_md), "entry 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x11000000), LogicalAddress::from(0xFFFF100000200000), 0x200000);
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1F800000), LogicalAddress::from(0xFFFF400000000000), 0x200000);
    check!(compare_metadata(page.entry_at(4), &expected_md), "entry 4 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to insert a free entry before another free entry which is
///   contiguous to it
///
/// ### Then
///
/// - The entry is merged with the next entry
/// - The returned excess is empty
pub(super) fn insert_free_entry_before_free_contiguous(page:&mut TracingPage) {
    scenario!("Insert a free entry before free entry (contiguous)");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    let mut new_md = Metadata::new_free(PhysicalAddress::from(0x1F700000), LogicalAddress::from(0xFFFF3FFFFFF00000), 0x100000);
    // When
    let insert_result = page.insert(new_md);
    // Then
    check!(insert_result.is_ok(), "failed to insert");
    check!(insert_result.get_or_panic().is_none(), "returned excess is not empty");
    check!(page.size() == 4, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF100000000000), 0x200000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x11000000), LogicalAddress::from(0xFFFF100000200000), 0x200000);
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1F700000), LogicalAddress::from(0xFFFF3FFFFFF00000), 0x300000);
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to insert a free entry before another free entry which is
///   not contiguous to it
///
/// ### Then
///
/// - The entry is not merged with the next and is inserted successfully
/// - The returned excess is empty
/// - The page is updated correctly
pub(super) fn insert_free_entry_before_free_non_contiguous(page:&mut TracingPage) {
    scenario!("Insert a free entry before free entry (non-contiguous)");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    let mut new_md = Metadata::new_free(PhysicalAddress::from(0x1F700000), LogicalAddress::from(0xFFFF300000000000), 0x100000);
    // When
    let insert_result = page.insert(new_md);
    // Then
    check!(insert_result.is_ok(), "failed to insert");
    check!(insert_result.get_or_panic().is_none(), "returned excess is not empty");
    check!(page.size() == 5, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF100000000000), 0x200000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x11000000), LogicalAddress::from(0xFFFF100000200000), 0x200000);
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    check!(compare_metadata(page.entry_at(3), &new_md), "entry 3 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1F800000), LogicalAddress::from(0xFFFF400000000000), 0x200000);
    check!(compare_metadata(page.entry_at(4), &expected_md), "entry 4 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to insert a free entry after another free entry which is
///   contiguous to it
///
/// ### Then
///
/// - The entry is merged with the previous entry
/// - The returned excess is empty
pub(super) fn insert_free_entry_after_free_contiguous(page:&mut TracingPage) {
    scenario!("Insert a free entry after free entry (contiguous)");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    let mut new_md = Metadata::new_free(PhysicalAddress::from(0x10400000), LogicalAddress::from(0xFFFF100000200000), 0x100000);
    // When
    let insert_result = page.insert(new_md);
    // Then
    check!(insert_result.is_ok(), "failed to insert");
    check!(insert_result.get_or_panic().is_none(), "returned excess is not empty");
    check!(page.size() == 4, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF100000000000), 0x300000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x11000000), LogicalAddress::from(0xFFFF100000200000), 0x200000);
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1F800000), LogicalAddress::from(0xFFFF400000000000), 0x200000);
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to insert a free entry after another free entry which is
///   not contiguous to it
///
/// ### Then
///
/// - The entry is not merged with the previous and is inserted successfully
/// - The returned excess is empty
/// - The page is updated correctly
pub(super) fn insert_free_entry_after_free_non_contiguous(page:&mut TracingPage) {
    scenario!("Insert a free entry after free entry (non-contiguous)");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    let mut new_md = Metadata::new_free(PhysicalAddress::from(0x10400000), LogicalAddress::from(0xFFFF300000000000), 0x100000);
    // When
    let insert_result = page.insert(new_md);
    // Then
    check!(insert_result.is_ok(), "failed to insert");
    check!(insert_result.get_or_panic().is_none(), "returned excess is not empty");
    check!(page.size() == 5, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF100000000000), 0x200000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    check!(compare_metadata(page.entry_at(2), &new_md), "entry 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x11000000), LogicalAddress::from(0xFFFF100000200000), 0x200000);
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1F800000), LogicalAddress::from(0xFFFF400000000000), 0x200000);
    check!(compare_metadata(page.entry_at(4), &expected_md), "entry 4 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to pop an entry
///
/// ### Then
///
/// - The last entry is popped succesfully
/// - The page is updated correctly
pub(super) fn pop_entry(page:&mut TracingPage) {
    scenario!("Try pop on page");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    // When
    let pop_result = page.try_pop();
    // Then
    check!(pop_result.is_ok(), "failed to pop");
    check!(page.size() == 3, "page size is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1F800000), LogicalAddress::from(0xFFFF400000000000), 0x200000);
    check!(compare_metadata(&pop_result.get_or_panic(), &expected_md), "popped entry is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF100000000000), 0x200000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x11000000), LogicalAddress::from(0xFFFF100000200000), 0x200000);
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::default();
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to extract an entry located at an index which is greater
///   than the current size of the array
///
/// ### Then
///
/// - The request is refused
/// - The page is not modified
pub(super) fn fail__extract_entry_out_of_bounds(page:&mut TracingPage) {
    fail_scenario!("Try extract an entry (out-of-bounds)");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    // When / Then
    check!(page.try_extract(9).is_err(), "result is Ok");
    // Then
    check!(page.size() == 4, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF100000000000), 0x200000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x11000000), LogicalAddress::from(0xFFFF100000200000), 0x200000);
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1F800000), LogicalAddress::from(0xFFFF400000000000), 0x200000);
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to extract an entry located at an index which corresponds
///   to the first entry of the array
///
/// ### Then
///
/// - The entry is extracted succesfully
/// - The page is updated correctly
pub(super) fn extract_entry_first(page:&mut TracingPage) {
    scenario!("Try extract an entry (first)");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    // When
    let extract_result = page.try_extract(0);
    // Then
    check!(extract_result.is_ok(), "failed to extract");
    check!(page.size() == 3, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(&extract_result.get_or_panic(), &expected_md), "extracted entry is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF100000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x11000000), LogicalAddress::from(0xFFFF100000200000), 0x200000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1F800000), LogicalAddress::from(0xFFFF400000000000), 0x200000);
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::default();
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to extract an entry located at an index which corresponds
///   to an entry in the middle of the array
///
/// ### Then
///
/// - The entry is extracted succesfully
/// - The page is updated correctly
pub(super) fn extract_entry_middle(page:&mut TracingPage) {
    scenario!("Try extract an entry (middle)");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    // When
    let extract_result = page.try_extract(1);
    // Then
    check!(extract_result.is_ok(), "failed to extract");
    check!(page.size() == 3, "page size is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF100000000000), 0x200000);
    check!(compare_metadata(&extract_result.get_or_panic(), &expected_md), "extracted entry is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x11000000), LogicalAddress::from(0xFFFF100000200000), 0x200000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1F800000), LogicalAddress::from(0xFFFF400000000000), 0x200000);
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::default();
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to extract an entry located at an index which corresponds
///   to the last non-none entry of the array
///
/// ### Then
///
/// - The entry is extracted succesfully
/// - The page is updated correctly
pub(super) fn extract_entry_last(page:&mut TracingPage) {
    scenario!("Try extract an entry (first)");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    // When
    let extract_result = page.try_extract(3);
    // Then
    check!(extract_result.is_ok(), "failed to extract");
    check!(page.size() == 3, "page size is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1F800000), LogicalAddress::from(0xFFFF400000000000), 0x200000);
    check!(compare_metadata(&extract_result.get_or_panic(), &expected_md), "extracted entry is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF100000000000), 0x200000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x11000000), LogicalAddress::from(0xFFFF100000200000), 0x200000);
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::default();
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to partially remove a free entry (the lower part of it)
///
/// ### Then
///
/// - The entry is updated correctly
/// - The returned reminder is zero
pub(super) fn remove_free_entry_partially_lower(page:&mut TracingPage) {
    scenario!("Try remove a free entry partially (lower)");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    // When
    let remove_result = page.try_remove(0x10200000.into(), 0x100000);
    // Then
    check!(remove_result.is_ok(), "failed to partially remove");
    check!(remove_result.get_or_panic().is_zero(), "returned reminder is different");
    check!(page.size() == 4, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10300000), LogicalAddress::from(0xFFFF100000100000), 0x100000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x11000000), LogicalAddress::from(0xFFFF100000200000), 0x200000);
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1F800000), LogicalAddress::from(0xFFFF400000000000), 0x200000);
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to partially remove a free entry (the higher part of it)
///
/// ### Then
///
/// - The entry is updated correctly
/// - The returned reminder is zero
pub(super) fn remove_free_entry_partially_higher(page:&mut TracingPage) {
    scenario!("Try remove a free entry partially (higher)");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    // When
    let remove_result = page.try_remove(0x10300000.into(), 0x100000);
    // Then
    check!(remove_result.is_ok(), "failed to partially remove");
    check!(remove_result.get_or_panic().is_zero(), "returned reminder is different");
    check!(page.size() == 4, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF100000000000), 0x100000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x11000000), LogicalAddress::from(0xFFFF100000200000), 0x200000);
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1F800000), LogicalAddress::from(0xFFFF400000000000), 0x200000);
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to partially remove a free entry (the middle part of it)
///
/// ### Then
///
/// - The entry is updated correctly
/// - Two new entries are created with the left-over size
/// - The returned reminder is zero
pub(super) fn remove_free_entry_partially_middle(page:&mut TracingPage) {
    scenario!("Try remove a free entry partially (middle)");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    // When
    let remove_result = page.try_remove(0x10280000.into(), 0x100000);
    // Then
    check!(remove_result.is_ok(), "failed to partially remove");
    check!(remove_result.get_or_panic().is_zero(), "returned reminder is different");
    check!(page.size() == 5, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF100000000000), 0x80000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10380000), LogicalAddress::from(0xFFFF100000180000), 0x80000);
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x11000000), LogicalAddress::from(0xFFFF100000200000), 0x200000);
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1F800000), LogicalAddress::from(0xFFFF400000000000), 0x200000);
    check!(compare_metadata(page.entry_at(4), &expected_md), "entry 4 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to entirely remove a free entry
///
/// ### Then
///
/// - The entry is removed succesfully
/// - The returned reminder is zero
pub(super) fn remove_free_entry_entirely(page:&mut TracingPage) {
    scenario!("Try remove a free entry entirely");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    // When
    let remove_result = page.try_remove(0x10200000.into(), 0x200000);
    // Then
    check!(remove_result.is_ok(), "failed to remove");
    check!(remove_result.get_or_panic().is_zero(), "returned reminder is different");
    check!(page.size() == 3, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x11000000), LogicalAddress::from(0xFFFF100000200000), 0x200000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1F800000), LogicalAddress::from(0xFFFF400000000000), 0x200000);
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::default();
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to partially remove a taken entry
///
/// ### Then
///
/// - The request is refused
/// - The page is not modified
pub(super) fn fail__remove_taken_entry_partially(page:&mut TracingPage) {
    fail_scenario!("Try remove a taken entry partially");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    // When / Then
    check!(page.try_remove(0x11000000.into(), 0x100000).is_err(), "result is Ok");
    // Then
    check!(page.size() == 4, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF100000000000), 0x200000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x11000000), LogicalAddress::from(0xFFFF100000200000), 0x200000);
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1F800000), LogicalAddress::from(0xFFFF400000000000), 0x200000);
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to entirely remove a taken entry
///
/// ### Then
///
/// - The entry is removed succesfully
/// - The returned reminder is zero
pub(super) fn remove_taken_entry_entirely(page:&mut TracingPage) {
    scenario!("Try remove a taken entry entirely");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    // When / Then
    let remove_result = page.try_remove(0x10000000.into(), 0x200000);
    // Then
    check!(remove_result.is_ok(), "failed to remove");
    check!(remove_result.get_or_panic().is_zero(), "returned reminder is different");
    check!(page.size() == 3, "page size is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF100000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x11000000), LogicalAddress::from(0xFFFF100000200000), 0x200000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1F800000), LogicalAddress::from(0xFFFF400000000000), 0x200000);
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::default();
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries which are not contiguous
///
/// ### When
///
/// - Requesting to remove an allocation bigger than one entry
///
/// ### Then
///
/// - The request is refused
/// - The page is not modified
pub(super) fn fail__remove_non_contiguous_entries_at_once(page:&mut TracingPage) {
    fail_scenario!("Try remove non-contiguous entries at once");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    // When / Then
    check!(page.try_remove(0x10000000.into(), 0x400000).is_err(), "first result is Ok");
    check!(page.try_remove(0x10200000.into(), 0x400000).is_err(), "second result is Ok");
    // Then
    check!(page.size() == 4, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF100000000000), 0x200000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x11000000), LogicalAddress::from(0xFFFF100000200000), 0x200000);
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1F800000), LogicalAddress::from(0xFFFF400000000000), 0x200000);
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries which are contiguous
///
/// ### When
///
/// - Requesting to remove an allocation bigger than one entry
///
/// ### Then
///
/// - The entries are removed successfully
/// - The returned reminder is zero
/// - The page is updated correctly
pub(super) fn remove_contiguous_entries_at_once(page:&mut TracingPage) {
    scenario!("Try remove contiguous entries at once");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_contiguous_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    // When
    let remove_result = page.try_remove(0x10000000.into(), 0x500000);
    // Then
    check!(remove_result.is_ok(), "failed to remove");
    check!(remove_result.get_or_panic().is_zero(), "returned reminder is different");
    check!(page.size() == 1, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10500000), LogicalAddress::from(0xFFFF000000500000), 0x100000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::default();
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to entirely take an already taken entry
///
/// ### Then
///
/// - The request is refused
/// - The page is not modified
pub(super) fn fail__take_taken_entry_entirely(page:&mut TracingPage) {
    fail_scenario!("Take an already taken entry entirely");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    // When / Then
    check!(page.take(0x10000000.into(), 0x200000).is_err(), "result is Ok");
    // Then
    check!(page.size() == 4, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF100000000000), 0x200000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x11000000), LogicalAddress::from(0xFFFF100000200000), 0x200000);
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1F800000), LogicalAddress::from(0xFFFF400000000000), 0x200000);
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to entirely take a free entry
///
/// ### Then
///
/// - The entry is updated correctly
/// - The returned excess is empty
pub(super) fn take_free_entry_entirely(page:&mut TracingPage) {
    scenario!("Take a free entry entirely");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    // When
    let take_result = page.take(0x10200000.into(), 0x200000);
    // Then
    check!(take_result.is_ok(), "failed to take");
    check!(take_result.get_or_panic().is_none(), "returned excess is not empty");
    check!(page.size() == 4, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF100000000000), 0x200000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x11000000), LogicalAddress::from(0xFFFF100000200000), 0x200000);
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1F800000), LogicalAddress::from(0xFFFF400000000000), 0x200000);
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to partially take an already taken entry
///
/// ### Then
///
/// - The request is refused
/// - The page is not modified
pub(super) fn fail__take_taken_entry_partially(page:&mut TracingPage) {
    fail_scenario!("Take an already taken entry partially");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    // When / Then
    check!(page.take(0x10000000.into(), 0x100000).is_err(), "result is Ok");
    // Then
    check!(page.size() == 4, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF100000000000), 0x200000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x11000000), LogicalAddress::from(0xFFFF100000200000), 0x200000);
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1F800000), LogicalAddress::from(0xFFFF400000000000), 0x200000);
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to partially take a free entry (the lower part of it)
///
/// ### Then
///
/// - The entry is updated correctly
/// - A new entry is created with the left-over size
/// - The returned excess is empty
/// - The array is updated correctly
pub(super) fn take_free_entry_partially_lower(page:&mut TracingPage) {
    scenario!("Take a free entry partially (lower)");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    // When
    let take_result = page.take(0x10200000.into(), 0x100000);
    // Then
    check!(take_result.is_ok(), "failed to take");
    check!(take_result.get_or_panic().is_none(), "returned excess is not empty");
    check!(page.size() == 5, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF100000000000), 0x100000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10300000), LogicalAddress::from(0xFFFF100000100000), 0x100000);
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x11000000), LogicalAddress::from(0xFFFF100000200000), 0x200000);
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1F800000), LogicalAddress::from(0xFFFF400000000000), 0x200000);
    check!(compare_metadata(page.entry_at(4), &expected_md), "entry 4 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to partially take a free entry (the higher part of it)
///
/// ### Then
///
/// - The entry is updated correctly
/// - A new entry is created with the left-over size
/// - The returned excess is empty
/// - The array is updated correctly
pub(super) fn take_free_entry_partially_higher(page:&mut TracingPage) {
    scenario!("Take a free entry partially (higher)");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    // When
    let take_result = page.take(0x10300000.into(), 0x100000);
    // Then
    check!(take_result.is_ok(), "failed to take");
    check!(take_result.get_or_panic().is_none(), "returned excess is not empty");
    check!(page.size() == 5, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF100000000000), 0x100000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10300000), LogicalAddress::from(0xFFFF100000100000), 0x100000);
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x11000000), LogicalAddress::from(0xFFFF100000200000), 0x200000);
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1F800000), LogicalAddress::from(0xFFFF400000000000), 0x200000);
    check!(compare_metadata(page.entry_at(4), &expected_md), "entry 4 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to partially take a free entry (the middle part of it)
///
/// ### Then
///
/// - The entry is updated correctly
/// - Two new entries are created with the left-over size
/// - The returned excess is empty
/// - The array is updated correctly
pub(super) fn take_free_entry_partially_middle(page:&mut TracingPage) {
    scenario!("Take a free entry partially (middle)");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    // When
    let take_result = page.take(0x10280000.into(), 0x100000);
    // Then
    check!(take_result.is_ok(), "failed to take");
    check!(take_result.get_or_panic().is_none(), "returned excess is not empty");
    check!(page.size() == 6, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF100000000000), 0x80000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10280000), LogicalAddress::from(0xFFFF100000080000), 0x100000);
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10380000), LogicalAddress::from(0xFFFF100000180000), 0x80000);
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x11000000), LogicalAddress::from(0xFFFF100000200000), 0x200000);
    check!(compare_metadata(page.entry_at(4), &expected_md), "entry 4 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1F800000), LogicalAddress::from(0xFFFF400000000000), 0x200000);
    check!(compare_metadata(page.entry_at(5), &expected_md), "entry 5 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to take some space whish is bigger than the corresponding free entry
///
/// ### Then
///
/// - The request is refused
/// - The page is not modified
pub(super) fn fail__take_free_entry_smaller(page:&mut TracingPage) {
    fail_scenario!("Take space bigger than a free entry");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    // When / Then
    check!(page.take(0x10200000.into(), 0x200001).is_err(), "result is Ok");
    check!(page.take(0x1F800000.into(), 0x200001).is_err(), "result is Ok");
    // Then
    check!(page.size() == 4, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF100000000000), 0x200000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x11000000), LogicalAddress::from(0xFFFF100000200000), 0x200000);
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1F800000), LogicalAddress::from(0xFFFF400000000000), 0x200000);
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to drop a free entry
///
/// ### Then
///
/// - The request is refused
/// - The page is not modified
pub(super) fn fail__drop_free_entry(page:&mut TracingPage) {
    fail_scenario!("Drop a free entry");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    // When / Then
    check!(page.drop(0x10200000.into()).is_err(), "result is Ok");
    // Then
    check!(page.size() == 4, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF100000000000), 0x200000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x11000000), LogicalAddress::from(0xFFFF100000200000), 0x200000);
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1F800000), LogicalAddress::from(0xFFFF400000000000), 0x200000);
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to drop a taken entry which is not contiguous to
///   any other free entry
///
/// ### Then
///
/// - The entry is updated correctly
/// - The entry is not merged with neighbour free entries
pub(super) fn drop_taken_entry_noncontiguous(page:&mut TracingPage) {
    scenario!("Drop a taken entry (non-contiguous)");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    // When / Then
    check!(page.drop(0x11000000.into()).is_ok(), "failed to drop");
    // Then
    check!(page.size() == 4, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF100000000000), 0x200000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x11000000), LogicalAddress::from(0xFFFF100000200000), 0x200000);
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1F800000), LogicalAddress::from(0xFFFF400000000000), 0x200000);
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to drop a taken entry which is followed by a free
///   entry that is contiguous to it
///
/// ### Then
///
/// - The entry is updated correctly
/// - The entry is merged with the following free entry
/// - The array is updated correctly
pub(super) fn drop_taken_entry_contiguous_right(page:&mut TracingPage) {
    scenario!("Drop a taken entry (contiguous, right)");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_contiguous_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    // When / Then
    check!(page.drop(0x10200000.into()).is_ok(), "failed to drop");
    // Then
    check!(page.size() == 3, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF000000200000), 0x300000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10500000), LogicalAddress::from(0xFFFF000000500000), 0x100000);
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::default();
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to drop a taken entry which is preceded by a free
///   entry that is contiguous to it
///
/// ### Then
///
/// - The entry is updated correctly
/// - The entry is merged with the preceding free entry
/// - The array is updated correctly
pub(super) fn drop_taken_entry_contiguous_left(page:&mut TracingPage) {
    scenario!("Drop a taken entry (contiguous, left)");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_contiguous_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    // When / Then
    check!(page.drop(0x10500000.into()).is_ok(), "failed to drop");
    // Then
    check!(page.size() == 3, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF000000200000), 0x200000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10400000), LogicalAddress::from(0xFFFF000000400000), 0x200000);
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::default();
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to drop a taken entry which is both preceded and followed
///   by a free entry that are contiguous to it
///
/// ### Then
///
/// - The entry is updated correctly
/// - The entry is merged with both the preceding and following free entries
/// - The array is updated correctly
pub(super) fn drop_taken_entry_contiguous_both(page:&mut TracingPage) {
    scenario!("Drop a taken entry (contiguous, both)");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_contiguous_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    page.entry_at_mut(0).set_free();
    // When / Then
    check!(page.drop(0x10200000.into()).is_ok(), "failed to drop");
    // Then
    check!(page.size() == 2, "page size is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x500000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10500000), LogicalAddress::from(0xFFFF000000500000), 0x100000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::default();
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to resize a free entry to a smaller size
///
/// ### Then
///
/// - The request is refused
/// - The page is not modified
pub(super) fn fail__resize_free_entry_smaller(page:&mut TracingPage) {
    fail_scenario!("Resize a free entry to a smaller size");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    // When / Then
    check!(page.resize(0x10200000.into(), 0x100000).is_err(), "result is Ok");
    // Then
    check!(page.size() == 4, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF100000000000), 0x200000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x11000000), LogicalAddress::from(0xFFFF100000200000), 0x200000);
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1F800000), LogicalAddress::from(0xFFFF400000000000), 0x200000);
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to resize a free entry to a bigger size
///
/// ### Then
///
/// - The request is refused
/// - The page is not modified
pub(super) fn fail__resize_free_entry_bigger(page:&mut TracingPage) {
    fail_scenario!("Resize a free entry to a bigger size");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    // When / Then
    check!(page.resize(0x10200000.into(), 0x300000).is_err(), "result is Ok");
    // Then
    check!(page.size() == 4, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF100000000000), 0x200000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x11000000), LogicalAddress::from(0xFFFF100000200000), 0x200000);
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1F800000), LogicalAddress::from(0xFFFF400000000000), 0x200000);
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to resize a taken entry, which is not contiguous its following
///   entry, to a smaller size
///
/// ### Then
///
/// - The entry is updated successfully
/// - A new free entry is created with the remaining size
/// - The returned reminder is zero
/// - The page is updated correctly
pub(super) fn resize_taken_entry_smaller_noncontiguous(page:&mut TracingPage) {
    scenario!("Resize a taken entry to a smaller size (non-contiguous)");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    // When
    let resize_result = page.resize(0x11000000.into(), 0x100000);
    // Then
    check!(resize_result.is_ok(), "failed to resize");
    check!(resize_result.get_or_panic().is_zero(), "returned reminder is different");
    check!(page.size() == 5, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF100000000000), 0x200000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x11000000), LogicalAddress::from(0xFFFF100000200000), 0x100000);
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x11100000), LogicalAddress::from(0xFFFF100000300000), 0x100000);
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1F800000), LogicalAddress::from(0xFFFF400000000000), 0x200000);
    check!(compare_metadata(page.entry_at(4), &expected_md), "entry 4 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to resize a taken entry, which is followed but not contiguous to
///   a free entry, to a bigger size
///
/// ### Then
///
/// - The request is refused
/// - The page is not modified
pub(super) fn fail__resize_taken_entry_bigger_noncontiguous(page:&mut TracingPage) {
    fail_scenario!("Resize a taken entry (bigger, non-contiguous free)");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    // When / Then
    check!(page.resize(0x11000000.into(), 0x300000).is_err(), "result is Ok");
    // Then
    check!(page.size() == 4, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF100000000000), 0x200000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x11000000), LogicalAddress::from(0xFFFF100000200000), 0x200000);
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1F800000), LogicalAddress::from(0xFFFF400000000000), 0x200000);
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to resize a taken entry, which is followed and contiguous to
///   a taken entry, to a smaller size
///
/// ### Then
///
/// - The entry is updated successfully
/// - A new free entry is created with the remaining size
/// - The returned reminder is zero
/// - The page is updated correctly
pub(super) fn resize_taken_entry_smaller_contiguous_taken(page:&mut TracingPage) {
    scenario!("Resize a taken entry (smaller, contiguous taken)");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_contiguous_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    // When
    let resize_result = page.resize(0x10000000.into(), 0x100000);
    // Then
    check!(resize_result.is_ok(), "failed to resize");
    check!(resize_result.get_or_panic().is_zero(), "returned reminder is different");
    check!(page.size() == 5, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x100000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10100000), LogicalAddress::from(0xFFFF000000100000), 0x100000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF000000200000), 0x200000);
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10400000), LogicalAddress::from(0xFFFF000000400000), 0x100000);
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10500000), LogicalAddress::from(0xFFFF000000500000), 0x100000);
    check!(compare_metadata(page.entry_at(4), &expected_md), "entry 4 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to resize a taken entry, which is followed and contiguous to
///   a taken entry, to a bigger size
///
/// ### Then
///
/// - The request is refused
/// - The page is not modified
pub(super) fn fail__resize_taken_entry_bigger_contiguous_taken(page:&mut TracingPage) {
    fail_scenario!("Resize a taken entry (bigger, contiguous taken)");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_contiguous_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    // When / Then
    check!(page.resize(0x10000000.into(), 0x300000).is_err(), "result is Ok");
    // Then
    check!(page.size() == 4, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF000000200000), 0x200000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10400000), LogicalAddress::from(0xFFFF000000400000), 0x100000);
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10500000), LogicalAddress::from(0xFFFF000000500000), 0x100000);
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to resize a taken entry, which is followed and contiguus to
///   a free entry, to a smaller size
///
/// ### Then
///
/// - The entry is updated successfully
/// - The remaining size is merged with the following entry
/// - The returned reminder is zero
/// - The page is updated correctly
pub(super) fn resize_taken_entry_smaller_contiguous_free(page:&mut TracingPage) {
    scenario!("Resize a taken entry (smaller, contiguous free)");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_contiguous_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    // When
    let resize_result = page.resize(0x10200000.into(), 0x100000);
    // Then
    check!(resize_result.is_ok(), "failed to resize");
    check!(resize_result.get_or_panic().is_zero(), "returned reminder is different");
    check!(page.size() == 4, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF000000200000), 0x100000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10300000), LogicalAddress::from(0xFFFF000000300000), 0x200000);
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10500000), LogicalAddress::from(0xFFFF000000500000), 0x100000);
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to resize a taken entry, which followed and contiguous to
///   a free entry, to a bigger size. The requested size equals the sum of
///   the sizes of the two entries.
///
/// ### Then
///
/// - The entry is updated successfully
/// - The free entry is completely drained
/// - The returned reminder is zero
/// - The page is updated correctly
pub(super) fn resize_taken_entry_bigger_contiguous_free_equal(page:&mut TracingPage) {
    scenario!("Resize a taken entry (bigger, contiguous free, equal)");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_contiguous_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    // When
    let resize_result = page.resize(0x10200000.into(), 0x300000);
    // Then
    check!(resize_result.is_ok(), "failed to resize");
    check!(resize_result.get_or_panic().is_zero(), "returned reminder is different");
    check!(page.size() == 3, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF000000200000), 0x300000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10500000), LogicalAddress::from(0xFFFF000000500000), 0x100000);
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::default();
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to resize a taken entry, which is followed and contiguous to
///   a free entry, to a bigger size. The requested size is smaller than the sum
///   of the sizes of the two entries.
///
/// ### Then
///
/// - The entry is updated successfully
/// - The free entry is partially drained
/// - The returned reminder is zero
/// - The page is updated correctly
pub(super) fn resize_taken_entry_bigger_contiguous_free_big(page:&mut TracingPage) {
    scenario!("Resize a taken entry (bigger, contiguous free, big)");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_contiguous_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    // When
    let resize_result = page.resize(0x10200000.into(), 0x280000);
    // Then
    check!(resize_result.is_ok(), "failed to resize");
    check!(resize_result.get_or_panic().is_zero(), "returned reminder is different");
    check!(page.size() == 4, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF000000200000), 0x280000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10480000), LogicalAddress::from(0xFFFF000000480000), 0x80000);
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10500000), LogicalAddress::from(0xFFFF000000500000), 0x100000);
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`] containing some entries
///
/// ### When
///
/// - Requesting to resize a taken entry, which is followed and contiguous to
///   a free entry, to a bigger size. The requested size is bigger than the sum
///   of the sizes of the two entries.
///
/// ### Then
///
/// - The request is refused
/// - The page is not modified
pub(super) fn fail__resize_taken_entry_bigger_contiguous_free_small(page:&mut TracingPage) {
    fail_scenario!("Resize a taken entry (bigger, contiguous free, small)");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_contiguous_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    // When / Then
    check!(page.resize(0x10200000.into(), 0x380000).is_err(), "result is Ok");
    // Then
    check!(page.size() == 4, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000);
    check!(compare_metadata(page.entry_at(0), &expected_md), "entry 0 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF000000200000), 0x200000);
    check!(compare_metadata(page.entry_at(1), &expected_md), "entry 1 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x10400000), LogicalAddress::from(0xFFFF000000400000), 0x100000);
    check!(compare_metadata(page.entry_at(2), &expected_md), "entry 2 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10500000), LogicalAddress::from(0xFFFF000000500000), 0x100000);
    check!(compare_metadata(page.entry_at(3), &expected_md), "entry 3 is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A [`TracingPage`], which last entry is taken
///
/// ### When
///
/// - Requesting to resize the last entry to a bigger size
///
/// ### Then
///
/// - The request is refused
/// - The page is not modified
pub(super) fn fail__resize_taken_entry_bigger_last(page:&mut TracingPage) {
    fail_scenario!("Resize a taken entry (bigger, last)");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    add_contiguous_entries(page);
    assume!(page.is_empty() == false, "page pre-conditions");
    // When / Then
    check!(page.resize(0x10500000.into(), 0x200000).is_err(), "result is Ok");
    // Then
    check!(page.size() == 4, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x10500000), LogicalAddress::from(0xFFFF000000500000), 0x100000);
    check!(compare_metadata(page.entry_at(3), &expected_md), "last entry is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A full [`TracingPage`]
///
/// ### When
///
/// - Calling [`TracingPage::is_empty`]
/// - Calling [`TracingPage::is_full`]
/// - Iterating its entry throught a [`MetadataIterator`]
///
/// ### Then
///
/// - [`TracingPage::is_empty`] returns `true`
/// - [`TracingPage::is_full`] returns `false`
/// - The iterator visits all the entries
pub(super) fn full_page_checks(page:&mut TracingPage) {
    scenario!("Checks on a full page");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    fill_page(page);
    // When / Then
    check!(page.is_empty() == false, "is_empty() returned false");
    // When / Then
    check!(page.is_full() == true, "is_full() returned true");
    // When / Then
    check!(page.iterate().count() == METADATA_ARRAY_SIZE, "not all entries have been visited");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A full [`TracingPage`]
///
/// ### When
///
/// - Requesting to push an entry
///
/// ### Then
///
/// - The entry is pushed successfully
/// - The last entry is popped and returned as excess
/// - The page is updated correctly
pub(super) fn push_on_full_page(page:&mut TracingPage) {
    scenario!("Try push on a full page");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    fill_page(page);
    assume!(page.is_full() == true, "page pre-conditions");
    let new_md = Metadata::new_taken(PhysicalAddress::from(0xF00), LogicalAddress::from(0x9F00), 0x100);
    // When
    let push_result = page.try_push(new_md);
    // Then
    check!(push_result.is_ok(), "failed to push");
    check!(page.is_full() == true, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000E00), LogicalAddress::from(0x1009E00), 0x100);
    check!(compare_metadata(&push_result.get_or_panic().unwrap(), &expected_md), "returned excess is different");
    check!(compare_metadata(page.entry_at(0), &new_md), "pushed entry is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000), LogicalAddress::from(0xA000), 0x100);
    check!(compare_metadata(page.entry_at(1), &expected_md), "second entry is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1000D00), LogicalAddress::from(0x1009D00), 0x100);
    check!(compare_metadata(page.entry_at(METADATA_ARRAY_SIZE-1), &expected_md), "last entry is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A full [`TracingPage`], which last entry is taken
///
/// ### When
///
/// - Requesting to append an entry
///
/// ### Then
///
/// - The entry is not appended and is returned as excess
/// - The page is not modified
pub(super) fn append_on_full_page(page:&mut TracingPage) {
    fail_scenario!("Try append on a full page");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    fill_page(page);
    assume!(page.is_full() == true, "page pre-conditions");
    page.entry_at_mut(METADATA_ARRAY_SIZE-1).set_taken();
    let new_md = Metadata::new_taken(PhysicalAddress::from(0x1001000), LogicalAddress::from(0x100A000), 0x100);
    // When
    let append_result = page.try_append(new_md);
    // Then
    check!(append_result.is_ok(), "failed to append");
    check!(compare_metadata(&append_result.get_or_panic().unwrap(), &new_md), "returned excess is different");
    check!(page.is_full() == true, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000), LogicalAddress::from(0xA000), 0x100);
    check!(compare_metadata(page.entry_at(0), &expected_md), "first entry is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000E00), LogicalAddress::from(0x1009E00), 0x100);
    check!(compare_metadata(page.entry_at(METADATA_ARRAY_SIZE-1), &expected_md), "last entry is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A full [`TracingPage`]
///
/// ### When
///
/// - Requesting to insert an entry in the middle of the page
///
/// ### Then
///
/// - The entry is inserted successfully
/// - The last entry is popped and returned as excess
/// - The page is updated correctly
pub(super) fn insert_on_full_page(page:&mut TracingPage) {
    scenario!("Insert on a full page");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    fill_page(page);
    assume!(page.is_full() == true, "page pre-conditions");
    let new_md = Metadata::new_taken(PhysicalAddress::from(0x2000), LogicalAddress::from(0xB000), 0x100);
    // When
    let insert_result = page.insert(new_md);
    // Then
    check!(insert_result.is_ok(), "failed to insert");
    check!(page.is_full() == true, "page size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000E00), LogicalAddress::from(0x1009E00), 0x100);
    check!(compare_metadata(&insert_result.get_or_panic().pop().unwrap(), &expected_md), "returned excess is different");
    check!(compare_metadata(page.entry_at(16), &new_md), "entry 16 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1000D00), LogicalAddress::from(0x1009D00), 0x100);
    check!(compare_metadata(page.entry_at(METADATA_ARRAY_SIZE-1), &expected_md), "last entry is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A full [`TracingPage`]
///
/// ### When
///
/// - Requesting to pop an entry
///
/// ### Then
///
/// - The last entry is popped successfully
/// - The page is updated correctly
pub(super) fn pop_on_full_page(page:&mut TracingPage) {
    scenario!("Try pop on full page");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    fill_page(page);
    assume!(page.is_full() == true, "page pre-conditions");
    // When
    let pop_result = page.try_pop();
    // Then
    check!(pop_result.is_ok(), "failed to pop");
    check!(page.is_full() == false, "page is still full");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000E00), LogicalAddress::from(0x1009E00), 0x100);
    check!(compare_metadata(&pop_result.get_or_panic(), &expected_md), "returned entry is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000), LogicalAddress::from(0xA000), 0x100);
    check!(compare_metadata(page.entry_at(0), &expected_md), "first entry is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1000D00), LogicalAddress::from(0x1009D00), 0x100);
    check!(compare_metadata(page.entry_at(METADATA_ARRAY_SIZE-2), &expected_md), "last entry is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A full [`TracingPage`]
///
/// ### When
///
/// - Requesting to remove one entry from the middle of the page
///
/// ### Then
///
/// - The entry is removed successfully
/// - The returned reminder is zero
/// - The page is updated correctly
pub(super) fn remove_on_full_page(page:&mut TracingPage) {
    scenario!("Try remove on full page");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    fill_page(page);
    assume!(page.is_full() == true, "page pre-conditions");
    // When
    let remove_result = page.try_remove(0xF0000.into(), 0x100);
    // Then
    check!(remove_result.is_ok(), "failed to remove");
    check!(remove_result.get_or_panic().is_zero(), "returned reminder is different");
    check!(page.is_full() == false, "page is still full");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000), LogicalAddress::from(0xA000), 0x100);
    check!(compare_metadata(page.entry_at(0), &expected_md), "first entry is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000E00), LogicalAddress::from(0x1009E00), 0x100);
    check!(compare_metadata(page.entry_at(METADATA_ARRAY_SIZE-2), &expected_md), "last entry is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A full [`TracingPage`]
///
/// ### When
///
/// - Requesting to partially remove a free entry (the middle part of it)
///
/// ### Then
///
/// - The entry is updated correctly
/// - Two new entries are created with the left-over size
/// - The last entry is returned as positive reminder
/// - The page is updated correctly
pub(super) fn remove_free_entry_partially_middle_on_full_page(page:&mut TracingPage) {
    scenario!("Try remove a free entry partially on full page (middle)");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    fill_page(page);
    assume!(page.is_full() == true, "page pre-conditions");
    // When
    let remove_result = page.try_remove(0x2140.into(), 0x80);
    // Then
    check!(remove_result.is_ok(), "failed to remove");
    let reminder_md = match remove_result.get_or_panic() {
        Reminder::Positive(md) => md,
        _ => Metadata::default(),
    };
    check!(page.is_full() == true, "page is not full");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000E00), LogicalAddress::from(0x1009E00), 0x100);
    check!(compare_metadata(&reminder_md, &expected_md), "returned reminder is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x2100), LogicalAddress::from(0xB100), 0x40);
    check!(compare_metadata(page.entry_at(17), &expected_md), "entry 17 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x21C0), LogicalAddress::from(0xB1C0), 0x40);
    check!(compare_metadata(page.entry_at(18), &expected_md), "entry 18 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1000D00), LogicalAddress::from(0x1009D00), 0x100);
    check!(compare_metadata(page.entry_at(METADATA_ARRAY_SIZE-1), &expected_md), "last entry is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A full [`TracingPage`]
///
/// ### When
///
/// - Requesting to partially remove an amount of space that starts from the
///   last entry but is greater than its size
///
/// ### Then
///
/// - The entry is removed successfully
/// - A negative reminder is returned with the left-over size
/// - The page is updated correctly
pub(super) fn remove_entries_from_last_with_reminder_on_full_page(page:&mut TracingPage) {
    scenario!("Try remove some space bigger than the last entry on full page");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    fill_page(page);
    assume!(page.is_full() == true, "page pre-conditions");
    // When
    let remove_result = page.try_remove(0x1000E00.into(), 0x200);
    // Then
    check!(remove_result.is_ok(), "failed to remove");
    let reminder_md = match remove_result.get_or_panic() {
        Reminder::Negative(md) => md,
        _ => Metadata::default(),
    };
    check!(page.is_full() == false, "page is still full");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000F00), LogicalAddress::from(0x1009F00), 0x100);
    check!(compare_metadata(&reminder_md, &expected_md), "returned reminder is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1000D00), LogicalAddress::from(0x1009D00), 0x100);
    check!(compare_metadata(page.entry_at(METADATA_ARRAY_SIZE-2), &expected_md), "last entry is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A full [`TracingPage`] in which all entries are contiguous
///
/// ### When
///
/// - Requesting to remove a size equal to the sum of the size of all the entries
///
/// ### Then
///
/// - The entries are removed successfully
/// - The returned reminder is zero
/// - The page is empty
pub(super) fn remove_all_entries_on_full_page(page:&mut TracingPage) {
    scenario!("Try remove all the entries on full page");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    fill_page(page);
    assume!(page.is_full() == true, "page pre-conditions");
    // When
    let remove_result = page.try_remove(0x1000.into(), 0xFFFF00);
    // Then
    check!(remove_result.is_ok(), "failed to remove");
    check!(remove_result.get_or_panic().is_zero(), "returned reminder is different");
    check!(page.is_empty() == true, "page is not empty");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A full [`TracingPage`]
///
/// ### When
///
/// - Requesting to partially take a free entry (half of it)
///
/// ### Then
///
/// - The entry is updated correctly
/// - A new entry is created with the left-over size
/// - The last entry is returned as excess
/// - The page is updated correctly
pub(super) fn take_free_entry_partially_half_on_full_page(page:&mut TracingPage) {
    scenario!("Take a free entry partially on full page (half)");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    fill_page(page);
    assume!(page.is_full() == true, "page pre-conditions");
    // When
    let take_result = page.take(0x2100.into(), 0x80);
    // Then
    check!(take_result.is_ok(), "failed to take");
    check!(page.is_full() == true, "page is not full");
    let mut excess = take_result.get_or_panic();
    check!(excess.size() == 1, "returned excess size is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000E00), LogicalAddress::from(0x1009E00), 0x100);
    check!(compare_metadata(&excess.pop().unwrap(), &expected_md), "returned excess is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x2100), LogicalAddress::from(0xB100), 0x80);
    check!(compare_metadata(page.entry_at(17), &expected_md), "entry 17 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x2180), LogicalAddress::from(0xB180), 0x80);
    check!(compare_metadata(page.entry_at(18), &expected_md), "entry 18 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1000D00), LogicalAddress::from(0x1009D00), 0x100);
    check!(compare_metadata(page.entry_at(METADATA_ARRAY_SIZE-1), &expected_md), "last entry is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A full [`TracingPage`]
///
/// ### When
///
/// - Requesting to partially take a free entry (half of it)
///
/// ### Then
///
/// - The entry is updated correctly
/// - Two new entries are created with the left-over size
/// - The last two entries are returned as excess
/// - The page is updated correctly
pub(super) fn take_free_entry_partially_middle_on_full_page(page:&mut TracingPage) {
    scenario!("Take a free entry partially on full page (middle)");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    fill_page(page);
    assume!(page.is_full() == true, "page pre-conditions");
    // When
    let take_result = page.take(0x2140.into(), 0x80);
    // Then
    check!(take_result.is_ok(), "failed to take");
    check!(page.is_full() == true, "page is not full");
    let mut excess = take_result.get_or_panic();
    check!(excess.size() == 2, "returned excess size is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1000D00), LogicalAddress::from(0x1009D00), 0x100);
    check!(compare_metadata(&excess.pop().unwrap(), &expected_md), "returned excess is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000E00), LogicalAddress::from(0x1009E00), 0x100);
    check!(compare_metadata(&excess.pop().unwrap(), &expected_md), "returned excess is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x2100), LogicalAddress::from(0xB100), 0x40);
    check!(compare_metadata(page.entry_at(17), &expected_md), "entry 17 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x2140), LogicalAddress::from(0xB140), 0x80);
    check!(compare_metadata(page.entry_at(18), &expected_md), "entry 18 is different");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x21C0), LogicalAddress::from(0xB1C0), 0x40);
    check!(compare_metadata(page.entry_at(19), &expected_md), "entry 19 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000C00), LogicalAddress::from(0x1009C00), 0x100);
    check!(compare_metadata(page.entry_at(METADATA_ARRAY_SIZE-1), &expected_md), "last entry is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A full [`TracingPage`]
///
/// ### When
///
/// - Requesting to entirely drop a taken entry which is preceded
///   and followed by two free entries that are both contiguous to it
///
/// ### Then
///
/// - The entry is updated correctly
/// - The entry is merged with the preceding and following free entries
/// - The array is updated correctly
pub(super) fn drop_taken_entry_contiguous_both_on_full_page(page:&mut TracingPage) {
    scenario!("Drop a taken entry on full page (contiguous, both)");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    fill_page(page);
    assume!(page.is_full() == true, "page pre-conditions");
    // When
    check!(page.drop(0x2000.into()).is_ok(), "failed to drop");
    // Then
    check!(page.is_full() == false, "page is still full");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1F00), LogicalAddress::from(0xAF00), 0x300);
    check!(compare_metadata(page.entry_at(15), &expected_md), "entry 15 is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000E00), LogicalAddress::from(0x1009E00), 0x100);
    check!(compare_metadata(page.entry_at(METADATA_ARRAY_SIZE-3), &expected_md), "last entry is different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A full [`TracingPage`], which last entry is taken
///
/// ### When
///
/// - Requesting to resize the last entry to a bigger size
///
/// ### Then
///
/// - The entry is updated correctly
/// - A negative reminder is returned
pub(super) fn resize_taken_entry_bigger_last_on_full_page(page:&mut TracingPage) {
    scenario!("Resize a taken entry on full page (bigger, last)");
    // Given
    clear_page(page);
    assume!(page.is_empty() == true, "page pre-conditions");
    fill_page(page);
    assume!(page.is_full() == true, "page pre-conditions");
    // When
    let resize_result = page.resize(0x1000E00.into(), 0x200);
    // Then
    check!(resize_result.is_ok(), "failed to resize");
    let reminder_md = match resize_result.get_or_panic() {
        Reminder::Negative(md) => md,
        _ => Metadata::default(),
    };
    check!(page.is_full() == true, "page is not full");
    let expected_md = Metadata::new_free(PhysicalAddress::from(0x1000F00), LogicalAddress::from(0x1009F00), 0x100);
    check!(compare_metadata(&reminder_md, &expected_md), "returned reminder is different");
    let expected_md = Metadata::new_taken(PhysicalAddress::from(0x1000E00), LogicalAddress::from(0x1009E00), 0x200);
    check!(compare_metadata(page.entry_at(METADATA_ARRAY_SIZE-1), &expected_md), "last entry is different");
    test_passed!();
    wait!();
}

}

fn get_page<'a>() -> &'a mut TracingPage {
    unsafe { &mut *TracingPage::from_table_entry(TracingPagesIterator::new(MemoryOwner::Kernel).next().unwrap()) }
}

fn clear_page(page:&mut TracingPage) {
   page.clear();
}

fn add_entries(page:&mut TracingPage) {
    let _ = page.append_unchecked(Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000));
    // physically contiguous, but not logically
    let _ = page.append_unchecked(Metadata::new_free(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF100000000000), 0x200000));
    // logically contiguous, but not physically
    let _ = page.append_unchecked(Metadata::new_taken(PhysicalAddress::from(0x11000000), LogicalAddress::from(0xFFFF100000200000), 0x200000));
    let _ = page.append_unchecked(Metadata::new_free(PhysicalAddress::from(0x1F800000), LogicalAddress::from(0xFFFF400000000000), 0x200000));
}

fn add_contiguous_entries(page:&mut TracingPage) {
    let _ = page.append_unchecked(Metadata::new_taken(PhysicalAddress::from(0x10000000), LogicalAddress::from(0xFFFF000000000000), 0x200000));
    let _ = page.append_unchecked(Metadata::new_taken(PhysicalAddress::from(0x10200000), LogicalAddress::from(0xFFFF000000200000), 0x200000));
    let _ = page.append_unchecked(Metadata::new_free(PhysicalAddress::from(0x10400000), LogicalAddress::from(0xFFFF000000400000), 0x100000));
    let _ = page.append_unchecked(Metadata::new_taken(PhysicalAddress::from(0x10500000), LogicalAddress::from(0xFFFF000000500000), 0x100000));
}

fn fill_page(page:&mut TracingPage) {
    // first -> status: taken, paddr: 0x1000, laddr: 0xA000, size: 0x100
    // second -> status: free, paddr: 0x1100, laddr: 0xA100, size: 0x100
    // ...
    // second-last -> status: free, paddr: 0x1000D00, laddr: 0x1009D00, size: 0x100
    // last -> status: taken, paddr: 0x1000E00, laddr: 0x1009E00, size: 0x100
    page.fill_alternate(PhysicalAddress::from(0x1000), LogicalAddress::from(0xA000), 0x100);
}
