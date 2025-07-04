use crate::memory::{MemoryError, MemoryOwner};
use crate::memory::address::*;
use crate::memory::paging;

use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;



#[allow(non_upper_case_globals)]
static mut
memory_guard : AtomicBool = AtomicBool::new(false);


/// Manages the allocation and de-allocation of memory
///
/// The [`Allocator`] locks the memory guard when created and un-locks it
/// when destroyed. For this reason, its lifetime shall be extremely short
/// and its usage is intended to be a one-shot, so that the memory guard
/// remains un-locked and available to be used by other processes.
///
/// ## Example
///
/// ```
/// /* some code */
/// {
///     let allocator = Allocator::new(); // <- memory guard locked
///     allocator.allocate(0x1000, MemoryOwner::User);
/// } // <- memory guard un-locked
/// /* some code */
/// ```
pub(in crate::memory)
struct Allocator {}

impl Allocator {
    /// Creates a new [`Allocator`] and spin-locks the memory guard
    pub(super)
    fn new() -> Self {
        while !Self::try_lock() {}
        Self {}
    }

    /// Attempts to lock the memory guard and suddenly returns the result
    ///
    /// ## Returns
    ///
    /// Returns `true` if the memory guard was un-locked and has been locked
    /// successfully, or `false` otherwise.
    fn try_lock() -> bool {
        unsafe {
            memory_guard.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_ok()
        }
    }

    /// Attempts to un-lock the memory guard and suddenly returns
    ///
    /// This function does not return anything. Each call to this function should
    /// be considered successful: if the memory guard was locked it has been un-locked,
    /// if it was already un-locked this call hasn't changed its state.
    fn try_unlock() {
        unsafe {
            let _ = memory_guard.compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst);
        }
    }

    /// Un-locks the memory guard and panics on failure
    ///
    /// ## Panics
    ///
    /// Panics if the memory guard was already un-locked.
    fn unlock() {
        unsafe {
            let unlocked = memory_guard.compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst);
            if let Err(_) = unlocked {
                crate::panic("Unlocking unlocked memory guard");
            }
        }
    }

    /// Allocates `size` Bytes of dynamic memory
    ///
    /// ## Returns
    ///
    /// Returns an [`Err`] containing the error any sub-process fails, otherwise
    /// returns an [`Ok`] containing the [`LogicalAddress`] of the allocated memory.
    pub(in crate::memory)
    fn allocate(&self, size:u64, owner:MemoryOwner) -> Result<LogicalAddress,MemoryError> {
        if let Some(addr) = paging::search::find_available_space(size, owner) {
            return paging::take::take_available_space(addr.physical, size, owner)
                .map_err(|e| MemoryError::PagingError(e))
                .and(Ok(addr.logical));
        }
        let (page_type,n_pages) = paging::utilities::suitable_pages(size);
        let flags = paging::Bitmap::from(owner) | paging::Bitmap::from(page_type);
        // TODO: find_available_space() may have returned None because there wasn't enough
        //  space, but it may be that some (or all) of that space can be used for this allocation,
        //  in case it is contiguous and at the end. Check if it can be used: if so, reduce the number
        //  of pages to insert accordingly, get the start address of the usabe space and use it to allocate
        match paging::insert::force_insert_pages(page_type, n_pages, flags, owner) {
            Err(e) => Err(MemoryError::from(e)),
            Ok(page_addr) => {
                paging::take::take_available_space(page_addr.physical, size, owner)
                    .map_err(|e| MemoryError::PagingError(e))
                    .and(Ok(page_addr.logical))
            },
        }
    }

    /// Allocates `size` Bytes of dynamic memory and zeroes it
    ///
    /// ## Returns
    ///
    /// Returns an [`Err`] containing the error any sub-process fails, otherwise
    /// returns an [`Ok`] containing the [`LogicalAddress`] of the allocated memory.
    pub(in crate::memory)
    fn allocate_zeroed(&self, size:u64, owner:MemoryOwner) -> Result<LogicalAddress,MemoryError> {
        self.allocate(size, owner)
            .and_then(|laddr| unsafe {
                super::memset(laddr.get(), 0, size);
                Ok(laddr)
            })
    }

    /// Re-allocates the allocation currently pointed to by `laddr` and re-sizes
    /// it to the requested `new_size`
    ///
    /// ## Note
    ///
    /// Three different cases can happen to be:
    /// - If the new size equals the old size, no re-allocation takes place and
    ///   the current allocation is left un-touched
    /// - If the new size is smaller than the old size, no re-allocation takes
    ///   place and the current allocation is shrunk inplace
    /// - If the new size is bigger than the old size, no re-allocation takes
    ///   place if there's enough free space contiguous to the current allocation
    ///   that allows to expand it inplace, otherwise the allocation is moved to
    ///   a new location and `new_size` Bytes are copied in it
    ///
    /// ## Returns
    ///
    /// Returns an [`Err`] containing the error any sub-process fails, otherwise
    /// returns an [`Ok`] containing the [`LogicalAddress`] of the new memory location.
    pub(in crate::memory)
    fn reallocate(&self, laddr:LogicalAddress, new_size:u64, owner:MemoryOwner) -> Result<LogicalAddress,MemoryError> {
        let (can_relocate,old_size) = paging::query::can_relocate_inplace(laddr, new_size, owner)
            .map_err(|e| MemoryError::PagingError(e))?;
        if can_relocate {
            return paging::update::relocate_inplace(laddr, new_size, owner)
                .map_err(|e| MemoryError::PagingError(e))
                .and(Ok(laddr));
        }
        let new_laddr = self.allocate(new_size, owner)?;
        unsafe {
            super::memcpy(new_laddr.get(), laddr.get(), old_size);
        }
        return paging::drop::drop_occupied_space(laddr, owner)
            .map_err(|e| MemoryError::PagingError(e))
            .map(|()| new_laddr)
    }

    /// De-allocates the memory pointed to by `laddr`
    ///
    /// ## Returns
    ///
    /// Returns an [`Err`] containing the error any sub-process fails, otherwise
    /// returns an empty [`Ok`].
    pub(in crate::memory)
    fn deallocate(&self, laddr:LogicalAddress, owner:MemoryOwner) -> Result<(),MemoryError> {
        return paging::drop::drop_occupied_space(laddr, owner)
            .map_err(|e| MemoryError::PagingError(e))
    }
}

impl Drop for Allocator {
    /// Un-locks the memory guard
    fn drop(&mut self) {
        Self::try_unlock();
    }
}
