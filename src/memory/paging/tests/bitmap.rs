use crate::test::*;

use crate::memory::MemoryOwner;
use crate::memory::paging::PageType;
use crate::memory::paging::bitmap::*;

/// # Tests
///
/// ### Pre-conditions
///
/// - The bitmasks for the addresses are initialized
///
/// ### Post-conditions
///
/// - None
pub(crate) fn run_all_tests() {
    module!("memory::paging::Bitmap");
    test::address_masks();
    test::from_bits();
    test::with_bits();
    test::without_bits();
    test::has_bits();
    test::supervisor_bit_on_kernel_user_defaults();
    test::page_size_bit_on_1g_2m_4k_pages();
}

mod test {

use super::*;

/// # Test
///
/// ### Given
///
/// - Initialized address masks
///
/// ### When
///
/// - Printing address masks on screen
///
/// ### Then
///
/// - The user can view the bits used by the processor
pub(super) fn address_masks() {
    scenario!("Address masks");
    // Given / When / Then
    unsafe {
        show_hex!("1 GiB page", BITMASK_ADDRESS_1G.try_into().unwrap());
        show_hex!("2 MiB page", BITMASK_ADDRESS_2M.try_into().unwrap());
        show_hex!("4 KiB page", BITMASK_ADDRESS_4K.try_into().unwrap());
    }
    wait!();
}

/// # Test
///
/// ### Given
///
/// - Some bits
///
/// ### When
///
/// - Creating a bitmap on those bits
///
/// ### Then
///
/// - The bitmap has the same bits
pub(super) fn from_bits() {
    scenario!("From bits");
    // Given
    let bits : u64 = 0b0100001000000000000000111111000001000000000000000001000000010011;
    // When
    let bitmap = Bitmap::from(bits);
    // Then
    test!(bitmap.bits() == bits, "Bitmap's bits are different");
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A series of increasing bits
/// - A zeroed bitmap
///
/// ### When
///
/// - The new bits are progressively added to the bitmap
///
/// ### Then
///
/// - The bitmap contains the new bit as well as all the previous bits
pub(super) fn with_bits() {
    scenario!("With bits");
    // Given
    let mut expected_bits : [u64;64] = [0;64];
    for i in 0..64 {
        for j in 0..i {
            expected_bits[i] |= 1 << j;
        }
    }
    let mut bits : u64 = 0b0000000000000000000000000000000000000000000000000000000000000000;
    let mut bitmap = Bitmap::new();
    // When
    for i in 0..64 {
        bitmap = bitmap.with_bits(bits);
        bits |= 1 << i;
        // Then
        check!(bitmap.bits() == expected_bits[i], "Bitmap's bits are different");
    }
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A series of decreasing bits
/// - A oned bitmap
///
/// ### When
///
/// - The new bits are progressively removed from the bitmap
///
/// ### Then
///
/// - The bitmap doesn't contain the new bit as well as all the previous bits
pub(super) fn without_bits() {
    scenario!("Without bits");
    // Given
    let mut expected_bits : [u64;64] = [0xFFFFFFFFFFFFFFFF;64];
    for i in 0..64 {
        for j in 0..i {
            expected_bits[i] ^= 1 << (63 - j);
        }
    }
    let mut bits : u64 = 0b0000000000000000000000000000000000000000000000000000000000000000;
    let mut bitmap = Bitmap::from(0b1111111111111111111111111111111111111111111111111111111111111111);
    // When
    for i in 0..64 {
        bitmap = bitmap.without_bits(bits);
        bits |= 1 << (63 - i);
        // Then
        check!(bitmap.bits() == expected_bits[i], "Bitmap's bits are different");
    }
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A bitmap with some bits set
///
/// ### When
///
/// - `has_bits()` is called with 1 bit present in the bitmap
/// - `has_bits()` is called with all the bits present in the bitmap
/// - `has_bits()` is called with all the bits present in the bitmap and 1 extra bit
///
/// ### Then
///
/// - `has_bits()` returns `true`
/// - `has_bits()` returns `true`
/// - `has_bits()` returns `false`
pub(super) fn has_bits() {
    scenario!("Has bits");
    // Given
    let bits1 : u64 = 0b1000000000000000000000000000000000000000000000000000000000000000;
    let bits2 : u64 = 0b1000000000000000000000000000000000000000000000000000000000000001;
    let bits3 : u64 = 0b1100000000000000000000000000000000000000000000000000000000000001;
    let bitmap = Bitmap::from(0b1000000000000000000000000000000000000000000000000000000000000001);
    // When / Then
    check!(bitmap.has_bits(bits1) == true, "Bitmap's bits are different");
    check!(bitmap.has_bits(bits2) == true, "Bitmap's bits are different");
    check!(bitmap.has_bits(bits3) == false, "Bitmap's bits are different");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A bitmap with default userland bits
/// - A bitmap with default kernelspace bits
///
/// ### When
///
/// - The **User/Supervisor** bit is checked
///
/// ### Then
///
/// - The bit is set for the default user bitmap
/// - The bit is not set for the default kernel bitmap
pub(super) fn supervisor_bit_on_kernel_user_defaults() {
    scenario!("Supervisor bit on Kernel and User defaults");
    // Given
    let flags_owner_user = Bitmap::from(MemoryOwner::User);
    let flags_owner_kernel = Bitmap::from(MemoryOwner::Kernel);
    // When / Then
    check!(flags_owner_user.supervised() == true, "No Supervisor bit for User");
    check!(flags_owner_kernel.supervised() == false, "Supervisor bit for Kernel");
    test_passed!();
    wait!();
}

/// # Test
///
/// ### Given
///
/// - A bitmap created for a 1GiB page
/// - A bitmap created for a 2MiB page
/// - A bitmap created for a 4KiB page
///
/// ### When
///
/// - The **Page Size** bit is checked
///
/// ### Then
///
/// - The bit is set for the bitmap of the 1GiB page
/// - The bit is set for the bitmap of the 2MiB page
/// - The bit is not set for the bitmap of the 4KiB page
pub(super) fn page_size_bit_on_1g_2m_4k_pages() {
    scenario!("Page size bit on 1GiB 2MiB and 4KiB pages");
    // Given
    let flags_page_1gib = Bitmap::from(PageType::OneGiB);
    let flags_page_2mib = Bitmap::from(PageType::TwoMiB);
    let flags_page_4kib = Bitmap::from(PageType::FourKiB);
    // When / Then
    check!(flags_page_1gib.page_size() == true, "No PageSize bit for 1GiB page");
    check!(flags_page_2mib.page_size() == true, "No PageSize bit for 2MiB page");
    check!(flags_page_4kib.page_size() == false, "PageSize bit for 4KiB page");
    test_passed!();
    wait!();
}

}
