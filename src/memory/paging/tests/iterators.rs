use crate::test::*;

use crate::memory::SIZE_8b;
use crate::memory::paging::{FIRST_ENTRY_INDEX,LIMIT_ENTRY_INDEX};
use crate::memory::paging::iterators::*;

/// # Tests
///
/// ### Pre-conditions
///
/// - None
///
/// ### Post-conditions
///
/// - None
pub(crate) fn run_indexers_tests() {
    module!("memory::paging::iterators", "indexers");
    test::page_table_entry_index_next();
    test::page_table_entry_index_get();
    test::page_table_entry_offset_next();
    test::page_table_entry_offset_get();
}

/// # Tests
///
/// ### Pre-conditions
///
/// - The memory map is initialized
/// - The paging structure of the process is initialized
/// - The tracing table of the process is initialized
/// - The allocation tables of the process are clean
///
/// ### Post-conditions
///
/// - None
pub(crate) fn run_tables_tests() {
    module!("memory::paging::iterators");
    //test::();
}

mod test {

use super::*;

/// # Test
///
/// ### Given
///
/// - An unbounded [`PageTableEntryIndex`]
///
/// ### When
///
/// - Calling [`Indexer::next`] repeatedly
///
/// ### Then
///
/// - All the indexes are correct at every step
/// - Calling [`Indexer::next`] again at the end returns [`None`]
pub(super) fn page_table_entry_index_next() {
    scenario!("Call Indexer::next() on PageTableEntryIndex");
    // Given
    let mut index = PageTableEntryIndex::unbounded();
    // When
    for expected_idx in FIRST_ENTRY_INDEX..LIMIT_ENTRY_INDEX {
        // Then
        let current_idx = index.next();
        check!(current_idx.is_some_and(|idx| idx == expected_idx), "Index is different");
    }
    // Then
    check!(index.next().is_none(), "Final index is Some");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - An unbounded [`PageTableEntryIndex`]
///
/// ### When
///
/// - Iterating with [`Indexer::next`]
/// - Calling [`Indexer::get`] at every step
///
/// ### Then
///
/// - All the indexes are correct at every step
/// - Calling [`Indexer::get`] again at the end returns [`None`]
pub(super) fn page_table_entry_index_get() {
    scenario!("Call Indexer::get() on PageTableEntryIndex");
    // Given
    let mut index = PageTableEntryIndex::unbounded();
    // When / Then
    let mut expected_idx = 0;
    let mut current_idx = index.get();
    check!(current_idx.is_some_and(|idx| idx == expected_idx), "Initial index is different");
    // When
    for expected_idx in FIRST_ENTRY_INDEX..LIMIT_ENTRY_INDEX {
        // Then
        assume!(index.next().is_some(), "Call to next() returned None");
        let current_idx = index.get();
        check!(current_idx.is_some_and(|idx| idx == expected_idx), "Index is different");
    }
    // Then
    assume!(index.next().is_none(), "Final call to next() returned Some");
    check!(index.get().is_none(), "Final index is Some");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - An unbounded [`PageTableEntryOffset`]
///
/// ### When
///
/// - Calling [`Indexer::next`] repeatedly
///
/// ### Then
///
/// - All the offsets are correct at every step
/// - Calling [`Indexer::next`] again at the end returns [`None`]
pub(super) fn page_table_entry_offset_next() {
    scenario!("Call Indexer::next() on PageTableEntryOffset");
    // Given
    let mut offset = PageTableEntryOffset::unbounded();
    // When
    let mut expected_ofs = 0;
    for _ in FIRST_ENTRY_INDEX..LIMIT_ENTRY_INDEX {
        // Then
        let current_ofs = offset.next();
        check!(current_ofs.is_some_and(|ofs| ofs == expected_ofs), "Offset is different");
        expected_ofs += SIZE_8b;
    }
    // Then
    check!(offset.next().is_none(), "Final offset is Some");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - An unbounded [`PageTableEntryOffset`]
///
/// ### When
///
/// - Iterating with [`Indexer::next`]
/// - Calling [`Indexer::get`] at every step
///
/// ### Then
///
/// - All the offsets are correct at every step
/// - Calling [`Indexer::get`] again at the end returns [`None`]
pub(super) fn page_table_entry_offset_get() {
    scenario!("Call Indexer::get() on PageTableEntryOffset");
    // Given
    let mut offset = PageTableEntryOffset::unbounded();
    // When / Then
    let mut expected_ofs = 0;
    let mut current_ofs = offset.get();
    check!(current_ofs.is_some_and(|ofs| ofs == expected_ofs), "Initial offset is different");
    // When
    for _ in FIRST_ENTRY_INDEX..LIMIT_ENTRY_INDEX {
        // Then
        assume!(offset.next().is_some(), "Call to next() returned None");
        let current_ofs = offset.get();
        check!(current_ofs.is_some_and(|ofs| ofs == expected_ofs), "Offset is different");
        expected_ofs += SIZE_8b;
    }
    // Then
    assume!(offset.next().is_none(), "Final call to next() returned Some");
    check!(offset.get().is_none(), "Final offset is Some");
    test_passed!();
    wait!();
}

}
