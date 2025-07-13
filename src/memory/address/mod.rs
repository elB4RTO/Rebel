pub(crate) mod logical;
pub(crate) mod physical;
pub(crate) mod total;

use crate::panic::*;

use core::ops::{Add, AddAssign, Sub, SubAssign};


pub(crate) use logical::LogicalAddress;
pub(crate) use physical::PhysicalAddress;
pub(in crate::memory) use total::TotalAddress;


pub(in crate::memory)
type AddressResult<A> = Result<A, AddressError>;


pub(in crate::memory)
trait Address : Sized + From<u64> + Add<u64> + AddAssign<u64> + Sub<u64> + SubAssign<u64> {
    /// Checks whether the address is null
    fn is_null(&self) -> bool;

    /// Returns the wrapped address
    fn get(&self) -> u64;

    /// Returns the wrapped address as `const` pointer
    fn as_ptr<T>(&self) -> *const T;

    /// Returns the wrapped address as `mut` pointer
    fn as_ptr_mut<T>(&self) -> *mut T;

    /// Dereferences the wrapped address and returns the data it points to as `const` reference
    ///
    /// ## Warning
    ///
    /// This function does not check whether the underlying address is valid.
    unsafe fn as_ref<T>(&self) -> &T {
        &*self.as_ptr::<T>()
    }

    /// Dereferences the wrapped address and returns the data it points to as `mut` reference
    ///
    /// ## Warning
    ///
    /// This function does not check whether the underlying address is valid.
    unsafe fn as_ref_mut<T>(&self) -> &mut T {
        &mut*self.as_ptr_mut::<T>()
    }

    /// Dereferences the wrapped address and returns a copy of the data it points to
    ///
    /// ## Warning
    ///
    /// This function does not check whether the underlying address is valid.
    /// Moreover, see [`core::ptr::read`] for safety concerns.
    unsafe fn read<T:Copy>(&self) -> T {
        self.as_ptr::<T>().read()
    }

    /// Dereferences the wrapped address and overwrites the data it points to
    ///
    /// ## Warning
    ///
    /// This function does not check whether the underlying address is valid.
    /// Moreover, see [`core::ptr::write`] for safety concerns.
    unsafe fn write<T>(&self, value:T) {
        self.as_ptr_mut::<T>().write(value);
    }
}


pub(in crate::memory)
trait Align<B:Sized + Into<u64>> {
    /// Checks whether the address is aligned with the given size
    ///
    /// ## Example
    ///
    /// ```
    /// let addr = address_from(0x10000);
    /// assert_eq!(addr.is_aligned(0x1000), true);
    /// assert_eq!(addr.is_aligned(0x20000), false);
    /// ```
    fn is_aligned(&self, bound:B) -> bool;

    /// Aligns the address to the lower bound of the given size
    ///
    /// If the address is already aligned, a call to this function has no effect
    ///
    /// ## Example
    ///
    /// ```
    /// let mut addr = address_from(0x201000);
    /// addr.align_to_lower(0x1000);
    /// assert_eq!(addr, address_from(0x201000));
    /// addr.align_to_lower(0x200000);
    /// assert_eq!(addr, address_from(0x200000));
    /// ```
    fn align_to_lower(&mut self, bound:B);

    /// As [`Align::align_to_lower()`], but takes ownership of `self` and returns it
    ///
    /// ## Example
    ///
    /// ```
    /// let addr = address_from(0x201000);
    /// let addr = addr.aligned_to_lower(0x1000);
    /// assert_eq!(addr, address_from(0x201000));
    /// let addr = addr.aligned_to_lower(0x200000);
    /// assert_eq!(addr, address_from(0x200000));
    /// ```
    fn aligned_to_lower(self, bound:B) -> Self;

    /// Aligns the address to the upper bound of the given size
    ///
    /// If the address is already aligned, a call to this function has no effect
    ///
    /// ## Example
    ///
    /// ```
    /// let mut addr = address_from(0x201000);
    /// addr.align_to_upper(0x1000);
    /// assert_eq!(addr, address_from(0x201000));
    /// addr.align_to_upper(0x200000);
    /// assert_eq!(addr, address_from(0x400000));
    /// ```
    fn align_to_upper(&mut self, bound:B);

    /// As [`Align::align_to_upper()`], but takes ownership of `self` and returns it
    ///
    /// ## Example
    ///
    /// ```
    /// let addr = address_from(0x201000);
    /// let addr = addr.aligned_to_upper(0x1000);
    /// assert_eq!(addr, address_from(0x201000));
    /// let addr = addr.aligned_to_upper(0x200000);
    /// assert_eq!(addr, address_from(0x400000));
    /// ```
    fn aligned_to_upper(self, bound:B) -> Self;

    /// Aligns the address to the upper bound of the given size
    ///
    /// Contrary to [`Align::align_to_upper()`], if the address is already aligned
    /// it will be forced to the next bound
    ///
    /// ## Example
    ///
    /// ```
    /// let mut addr = address_from(0x200000);
    /// addr.align_to_upper(0x100000);
    /// assert_eq!(addr, address_from(0x200000));
    /// addr.force_align_to_upper(0x100000);
    /// assert_eq!(addr, address_from(0x300000));
    /// ```
    fn force_align_to_upper(&mut self, bound:B);

    /// As [`Align::force_align_to_upper()`], but takes ownership of `self` and returns it
    ///
    /// ## Example
    ///
    /// ```
    /// let addr = address_from(0x200000);
    /// let addr = addr.aligned_to_upper(0x200000);
    /// assert_eq!(addr, address_from(0x200000));
    /// let addr = addr.force_aligned_to_upper(0x200000);
    /// assert_eq!(addr, address_from(0x400000));
    /// ```
    fn force_aligned_to_upper(self, bound:B) -> Self;
}


pub(crate)
enum AddressError {
    NullAddress,
    PhysicalToLogical,
    LogicalToPhysical,
}

impl Panic for AddressError {
    fn panic(&self) -> ! {
        use AddressError::*;
        match self {
            NullAddress       => panic("AddressError: NullAddress"),
            PhysicalToLogical => panic("AddressError: PhysicalToLogical"),
            LogicalToPhysical => panic("AddressError: LogicalToPhysical"),
        }
    }
}
