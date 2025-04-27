pub(crate) mod address;
pub(crate) mod allocator;
pub(in crate::memory) mod info;
pub(in crate::memory) mod map;
pub(crate) mod paging;
pub(crate) mod setup;

#[cfg(feature="unit_tests")]
pub(crate) mod tests;

pub(crate) use setup::init;
pub(crate) use paging::init_kernel_tracing_pages;

use address::*;
use allocator::Allocator;
use paging::PageType;

use crate::GetOrPanic;
use crate::panic::*;

use core::arch::global_asm;


// Sizes in Bytes
pub(in crate::memory) const
SIZE_1GiB   : u64 = 0x40000000;
pub(in crate::memory) const
SIZE_2MiB   : u64 = 0x200000;
pub(in crate::memory) const
SIZE_4KiB   : u64 = 0x1000;
pub(in crate::memory) const
SIZE_8b     : u64 = 0x8;
pub(in crate::memory) const
SIZE_1B     : u64 = 0x1;


// 64 bits
const
NATIVE_ALIGNMENT : u64 = 0x8;


// Starting memory address of the kernel's binary in-memory
pub(in crate::memory) const
KERNEL_PADDR_BASE           : u64 = 0x0000000001200000;
// Ending memory address of the kernel's binary in-memory
pub(in crate::memory) const
KERNEL_PADDR_LIMIT          : u64 = KERNEL_PADDR_BASE + KERNEL_CODE_SIZE - 0x1;

// Starting memory address of the kernel's stack
pub(in crate::memory) const
KERNEL_STACK_PADDR_BASE     : u64 = KERNEL_PADDR_BASE - 0x1;
// Ending memory address of the kernel's stack
pub(in crate::memory) const
KERNEL_STACK_PADDR_LIMIT    : u64 = KERNEL_PADDR_BASE - KERNEL_STACK_SIZE;


/// The amount of memory needed to store the kernel in memory
pub(in crate::memory) const
KERNEL_CODE_SIZE    : u64 = 0x4000000; // 64 MiB


/// The size of the kernel's stack
pub(in crate::memory) const
KERNEL_STACK_SIZE   : u64 = 0x1000000; // 16 MiB

/// The size of the other processes' stack
#[cfg(feature="huge_stack")]
pub(in crate::memory) const
USER_STACK_SIZE     : u64 = 0x1000000; // 16 MiB
#[cfg(not(feature="huge_stack"))]
pub(in crate::memory) const
USER_STACK_SIZE     : u64 = 0x800000;  //  8 MiB


/// Used to initialize something in an invalid state
pub(in crate::memory)
trait Invalid {
    /// Creates an invalid instance of the object
    fn invalid() -> Self;
}

/// Used to initialize something from a memory address
pub(in crate::memory)
trait Init<P> {
    /// Creates and writes an instance of the object in the memory location
    /// pointed to by `ptr`
    fn init(ptr:P);
}

/// Used to cast something from a memory address
pub(in crate::memory)
trait Cast<P> {
    /// Casts the content of the memory pointed to by `ptr` into an instance
    /// of the object and returns an constant pointer to it
    fn cast(ptr:P) -> *const Self;
    /// Casts the content of the memory pointed to by `ptr` into an instance
    /// of the object and returns an mutable pointer to it
    fn cast_mut(ptr:P) -> *mut Self;
}


/// Represents a memory owner
#[derive(Clone,Copy)]
pub(crate)
enum MemoryOwner {
    Kernel,
    User,
}

impl From<paging::Bitmap> for MemoryOwner {
    /// Creates a new [`MemoryOwner`] based on the supervisor bit
    /// of the given [`Bitmap`]
    fn from(bits:paging::Bitmap) -> Self {
        match bits.supervised() {
            false => MemoryOwner::User,
            true  => MemoryOwner::Kernel,
        }
    }
}


pub(crate)
enum MemoryError {
    /// Not enough memory
    NoMemory,
    /// Cannot find the requested memory
    NotFound,
    /// Attempt to request memory that has a different owner
    OwnershipMismatch,
    /// The address is not properly aligned
    UnalignedAddress,
    /// Attempt to take a slice of memory which is not free to take
    TakingNonFreeMemory,
    /// Attempt to drop a slice of memory which is free already
    DroppingFreeMemory,
    /// Attempt to deal with a slice of memory which is reserved
    ReservedMemory,
    /// Attempt to bring space from a slice of memory which is free
    BringingFreeSpace,
    /// Attempt to release space from a slice of memory which is free
    ReleasingFreeSpace,
    /// Attempt to deal with the space of a slice of memory which is reserved
    ReservedSpace,
    /// Attempt to deal with the space of a memory slice in a wrong way
    SpaceIssue,
    /// Error regarding paging
    PagingError(paging::PagingError),
    /// The request is not valid (to be only used when no other variant better applies)
    InvalidRequest,
}

impl From<paging::PagingError> for MemoryError {
    fn from(e:paging::PagingError) -> Self {
        Self::PagingError(e)
    }
}

impl Panic for MemoryError {
    fn panic(&self) -> ! {
        use MemoryError::*;
        match self {
            NoMemory            => panic("MemoryError: NoMemory"),
            NotFound            => panic("MemoryError: NotFound"),
            OwnershipMismatch   => panic("MemoryError: OwnershipMismatch"),
            UnalignedAddress    => panic("MemoryError: UnalignedAddress"),
            TakingNonFreeMemory => panic("MemoryError: TakingNonFreeMemory"),
            DroppingFreeMemory  => panic("MemoryError: DroppingFreeMemory"),
            ReservedMemory      => panic("MemoryError: ReservedMemory"),
            BringingFreeSpace   => panic("MemoryError: BringingFreeSpace"),
            ReleasingFreeSpace  => panic("MemoryError: ReleasingFreeSpace"),
            ReservedSpace       => panic("MemoryError: ReservedSpace"),
            SpaceIssue          => panic("MemoryError: SpaceIssue"),
            PagingError(_)      => panic("MemoryError: PagingError"),
            InvalidRequest      => panic("MemoryError: InvalidRequest"),
        }
    }
}


// TODO: Implement the GlobalAllocator trait?
/*#[global_allocator]
static ALLOCATOR : Allocator = Allocator::new();*/


global_asm!(include_str!("mem.asm"));

extern "C" {
    /// Sets `size` Bytes of `dst` to `val`
    pub(crate)
    fn memset(dst:u64, val:u8, size:u64);

    /// Copies the value of `size` Bytes from `src` into `dst`
    pub(crate)
    fn memcpy(dst:u64, src:u64, size:u64);

    /// Compares the value of `size` Bytes between `addr1` and `addr2`
    pub(crate)
    fn memcmp(addr1:u64, addr2:u64, size:u64) -> bool;

    /// Works as `memcpy` but is safer since it checks whether the two
    /// memory locations overlaps and copy data accordingly
    pub(crate)
    fn safe_copy(dst:u64, src:u64, size:u64);
}

/// Copies `n` times the Bytes of the default value of `T` into `dst`,
/// increasing `dst` by the size of `T` each time
///
/// ## Returns
///
/// Returns a pointer to `dst` casted as `T`.
pub(in crate::memory) unsafe
fn memset_defaulted<T:Default>(dst:u64, n:u64) -> *mut T {
    let element_size = core::mem::size_of::<T>() as u64;
    let elem = T::default();
    let src_addr = (&elem as *const T) as u64;
    let mut dst_addr = dst;
    for _ in 0..n {
        memcpy(dst_addr, src_addr, element_size);
        dst_addr += element_size;
    }
    return (dst as *mut u64).cast::<T>();
}


/// Aligns `value` to [`NATIVE_ALIGNMENT`] and returns it
fn aligned_to_native(value:u64) -> u64 {
    match value % NATIVE_ALIGNMENT {
        0 => value,
        r => value + (NATIVE_ALIGNMENT - r),
    }
}


/// Allocates `size` Bytes
///
/// ## Returns
///
/// Returns an [`Ok`] containing the [`LogicalAddress`] of the allocation
/// is successful, otherwise returns an [`Err`] containing the error.
pub(crate)
fn alloc(size:u64, owner:MemoryOwner) -> Result<LogicalAddress,MemoryError> {
    Allocator::new().allocate(aligned_to_native(size), owner)
        .map(|laddr| match laddr.is_aligned(NATIVE_ALIGNMENT) {
            false => MemoryError::UnalignedAddress.panic(),
            true => laddr,
        })
}

/// Allocates `size` Bytes and zeroes them
///
/// ## Returns
///
/// Returns an [`Ok`] containing the [`LogicalAddress`] of the allocation
/// is successful, otherwise returns an [`Err`] containing the error.
pub(crate)
fn zalloc(size:u64, owner:MemoryOwner) -> Result<LogicalAddress,MemoryError> {
    Allocator::new().allocate_zeroed(aligned_to_native(size), owner)
        .map(|laddr| match laddr.is_aligned(NATIVE_ALIGNMENT) {
            false => MemoryError::UnalignedAddress.panic(),
            true => laddr,
        })
}

/// Re-allocates the allocation in `laddr` with a size of `new_size`
///
/// ## Returns
///
/// Returns an [`Ok`] containing the [`LogicalAddress`] of the allocation
/// is successful, otherwise returns an [`Err`] containing the error.
pub(crate)
fn realloc(laddr:LogicalAddress, new_size:u64, owner:MemoryOwner) -> Result<LogicalAddress,MemoryError> {
    if !laddr.is_aligned(NATIVE_ALIGNMENT) {
        return Err(MemoryError::UnalignedAddress);
    }
    Allocator::new().reallocate(laddr, aligned_to_native(new_size), owner)
}

/// De-allocates the allocation in `laddr` with a size of `size`
///
/// ## Returns
///
/// Returns an [`Ok`] containing the [`LogicalAddress`] of the allocation
/// is successful, otherwise returns an [`Err`] containing the error.
pub(crate)
fn dealloc(laddr:LogicalAddress, size:u64, owner:MemoryOwner) -> Result<(),MemoryError> {
    if !laddr.is_aligned(NATIVE_ALIGNMENT) {
        return Err(MemoryError::UnalignedAddress);
    }
    Allocator::new().deallocate(laddr, aligned_to_native(size), owner)
}
