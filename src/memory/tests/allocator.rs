use crate::test::*;
use crate::memory::paging::tests::{check_pages, erase_pages};

use crate::{GetOrPanic, OkOrPanic};
use crate::memory::{alloc, dealloc, MemoryOwner};
use crate::memory::{SIZE_1B, SIZE_4KiB, SIZE_2MiB, SIZE_1GiB};
use crate::memory::address::Address;
use crate::memory::paging;
use crate::tty::*;

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
/// - All the allocated memory has been freed
/// - The allocation tables of the process are clean
pub(crate) fn run() {
    module!("memory::Allocator");
    test::allocate_deallocate_1B();
    erase_pages();
    test::allocate_deallocate_1B_twice();
    erase_pages();
    test::allocate_deallocate_4KiB();
    erase_pages();
    test::allocate_deallocate_4KiB_twice();
    erase_pages();
    test::allocate_deallocate_2MiB();
    erase_pages();
    test::allocate_deallocate_2MiB_twice();
    erase_pages();
    test::allocate_deallocate_128MiB();
    erase_pages();
    test::allocate_deallocate_512MiB();
    erase_pages();
}

mod test {

use super::*;

/// # Test
///
/// ### Given
///
/// - A clean paging structure with no table or page
///
/// ### When
///
/// - Requesting to allocate 1B of memory and suddenly deallocate it
///
/// ### Then
///
/// - One new 4KiB page is created
/// - The memory is succesfully allocated and deallocated
/// - No page is deleted
pub(super) fn allocate_deallocate_1B() {
    scenario!("Allocate 1 B . Deallocate 1 B");
    // Given
    assume!(check_pages(0,0,0,0,0,0), "pages pre-conditions");
    // When
    let laddr = alloc(SIZE_1B, MemoryOwner::Kernel).get_or_panic();
    dealloc(laddr, MemoryOwner::Kernel).ok_or_panic();
    // Then
    show_hex!("LADDR", laddr as usize);
    test!(check_pages(1,1,1,0,0,1), "paging structure mismatch");
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
/// - Requesting to allocate 1B of memory twice and suddenly deallocate both
///
/// ### Then
///
/// - One new 4KiB page is created
/// - The memory is succesfully allocated and deallocated
/// - No page is deleted
pub(super) fn allocate_deallocate_1B_twice() {
    scenario!("Allocate 1 B twice . Deallocate 1 B twice");
    // Given
    assume!(check_pages(0,0,0,0,0,0), "pages pre-conditions");
    // When
    let laddr1 = alloc(SIZE_1B, MemoryOwner::Kernel).get_or_panic();
    let laddr2 = alloc(SIZE_1B, MemoryOwner::Kernel).get_or_panic();
    dealloc(laddr1, MemoryOwner::Kernel).ok_or_panic();
    dealloc(laddr2, MemoryOwner::Kernel).ok_or_panic();
    // Then
    show_hex!("LADDR 1", laddr1 as usize);
    show_hex!("LADDR 2", laddr2 as usize);
    check!(laddr1 != laddr2, "addresses are identical");
    test!(check_pages(1,1,1,0,0,1), "paging structure mismatch");
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
/// - Requesting to allocate 4KiB of memory and suddenly deallocate it
///
/// ### Then
///
/// - One new 4KiB page is created
/// - The memory is succesfully allocated and deallocated
/// - No page is deleted
pub(super) fn allocate_deallocate_4KiB() {
    scenario!("Allocate 4 KiB . Deallocate 4 KiB");
    // Given
    assume!(check_pages(0,0,0,0,0,0), "pages pre-conditions");
    // When
    let laddr = alloc(SIZE_4KiB, MemoryOwner::Kernel).get_or_panic();
    dealloc(laddr, MemoryOwner::Kernel).ok_or_panic();
    // Then
    show_hex!("LADDR", laddr as usize);
    test!(check_pages(1,1,1,0,0,1), "paging structure mismatch");
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
/// - Requesting to allocate 4KiB of memory twice and suddenly deallocate both
///
/// ### Then
///
/// - Two new 4KiB pages are created
/// - The memory is succesfully allocated and deallocated
/// - The pages are not deleted
pub(super) fn allocate_deallocate_4KiB_twice() {
    scenario!("Allocate 4 KiB twice . Deallocate 4 KiB twice");
    // Given
    assume!(check_pages(0,0,0,0,0,0), "pages pre-conditions");
    // When
    let laddr1 = alloc(SIZE_4KiB, MemoryOwner::Kernel).get_or_panic();
    let laddr2 = alloc(SIZE_4KiB, MemoryOwner::Kernel).get_or_panic();
    dealloc(laddr1, MemoryOwner::Kernel).ok_or_panic();
    dealloc(laddr2, MemoryOwner::Kernel).ok_or_panic();
    // Then
    show_hex!("LADDR 1", laddr1 as usize);
    show_hex!("LADDR 2", laddr2 as usize);
    check!(laddr1 != laddr2, "addresses are identical");
    test!(check_pages(1,1,1,0,0,2), "paging structure mismatch");
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
/// - Requesting to allocate 2MiB of memory and suddenly deallocate it
///
/// ### Then
///
/// - One new 2MiB huge page is created
/// - The memory is succesfully allocated and deallocated
/// - No page is deleted
pub(super) fn allocate_deallocate_2MiB() {
    scenario!("Allocate 2 MiB . Deallocate 2 MiB");
    // Given
    assume!(check_pages(0,0,0,0,0,0), "pages pre-conditions");
    // When
    let laddr = alloc(SIZE_2MiB, MemoryOwner::Kernel).get_or_panic();
    dealloc(laddr, MemoryOwner::Kernel).ok_or_panic();
    // Then
    show_hex!("LADDR", laddr as usize);
    test!(check_pages(1,1,0,0,1,0), "paging structure mismatch");
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
/// - Requesting to allocate 2MiB of memory twice and suddenly deallocate it
///
/// ### Then
///
/// - Two new 2MiB huge pages are created
/// - The memory is succesfully allocated and deallocated
/// - No page is deleted
pub(super) fn allocate_deallocate_2MiB_twice() {
    scenario!("Allocate 2 MiB twice . Deallocate 2 MiB twice");
    // Given
    assume!(check_pages(0,0,0,0,0,0), "pages pre-conditions");
    // When
    let laddr1 = alloc(SIZE_2MiB, MemoryOwner::Kernel).get_or_panic();
    let laddr2 = alloc(SIZE_2MiB, MemoryOwner::Kernel).get_or_panic();
    dealloc(laddr1, MemoryOwner::Kernel).ok_or_panic();
    dealloc(laddr2, MemoryOwner::Kernel).ok_or_panic();
    // Then
    show_hex!("LADDR 1", laddr1 as usize);
    show_hex!("LADDR 2", laddr2 as usize);
    check!(laddr1 != laddr2, "addresses are identical");
    test!(check_pages(1,1,0,0,2,0), "paging structure mismatch");
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
/// - Requesting to allocate 128 MiB of memory and suddenly deallocate it
///
/// ### Then
///
/// - Sixtyfour new 2 MiB huge pages are created
/// - The memory is succesfully allocated and deallocated
/// - No page is deleted
pub(super) fn allocate_deallocate_128MiB() {
    scenario!("Allocate 128 MiB . Deallocate 128 MiB");
    // Given
    assume!(check_pages(0,0,0,0,0,0), "pages pre-conditions");
    // When
    let size_128mib = SIZE_2MiB * 64;
    let laddr = alloc(size_128mib, MemoryOwner::Kernel).get_or_panic();
    dealloc(laddr, MemoryOwner::Kernel).ok_or_panic();
    // Then
    show_hex!("LADDR", laddr as usize);
    test!(check_pages(1,1,0,0,64,0), "paging structure mismatch");
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
/// - Requesting to allocate 512 MiB of memory and suddenly deallocate it
///
/// ### Then
///
/// - One new 1 GiB huge page is created
/// - The memory is succesfully allocated and deallocated
/// - No page is deleted
pub(super) fn allocate_deallocate_512MiB() {
    scenario!("Allocate 512 MiB . Deallocate 512 MiB");
    // Given
    assume!(check_pages(0,0,0,0,0,0), "pages pre-conditions");
    // When
    let size_512mib = SIZE_2MiB * 256;
    let laddr = alloc(size_512mib, MemoryOwner::Kernel).get_or_panic();
    dealloc(laddr, MemoryOwner::Kernel).ok_or_panic();
    // Then
    show_hex!("LADDR", laddr as usize);
    test!(check_pages(1,0,0,1,0,0), "paging structure mismatch");
    wait!();
}

}
