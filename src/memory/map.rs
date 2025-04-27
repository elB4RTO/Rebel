use super::*;
use crate::memory::address::*;
use crate::memory::paging::*;


pub(super) const SLICE_SIZE_U16    : u16   = 4096;
pub(super) const SLICE_SIZE_U64    : u64   = 4096;
pub(super) const SLICE_SIZE_USIZE  : usize = 4096;


static mut LAST_RESERVED_SLICE : u64 = KERNEL_PADDR_LIMIT / SLICE_SIZE_U64;


/// The global memory map, which summarizes the whole memory from a high-level
/// perspective
///
/// Its purpose is to allow quick lookups over portions of the memory
#[allow(non_upper_case_globals)]
static mut memory_map : MemoryMap = MemoryMap::new();


/// Represents a slice of `4 KiB` of memory
///
/// The `KernelSpace` and `UserLand` variants store the amount
/// of memory still available in the slice
enum MemorySlice {
    Reserved,
    KernelSpace(u16),
    UserLand(u16),
    Free,
}

impl MemorySlice {
    /// Checks whether the memory slice is of `Reserved` type
    fn is_reserved(&self) -> bool {
        match self {
            Self::Reserved => true,
            _ => false,
        }
    }

    /// Checks whether the memory slice is of `Free` type
    fn is_free(&self) -> bool {
        match self {
            Self::Free => true,
            _ => false,
        }
    }

    /// Returns the available space left in the memory slice
    ///
    /// ## Returns
    ///
    /// Returns the contained value if the [`MemorySlice`] is of type
    /// [`MemorySlice::KernelSpace`] or [`MemorySlice::UserLand`], the
    /// maximum size of the type is [`MemorySlice::Free`] and `0` if
    /// the type is [`MemorySlice::::Reserved`].
    fn available_space(&self) -> u16 {
        use MemorySlice::*;
        match self {
            Reserved => 0_u16,
            Free => SLICE_SIZE_U16,
            KernelSpace(s)|UserLand(s) => *s,
        }
    }

    /// Returns the occupied space in the memory slice
    ///
    /// ## Returns
    ///
    /// Returns the computed occupied size if the [`MemorySlice`] is
    /// of type [`MemorySlice::KernelSpace`] or [`MemorySlice::UserLand`],
    /// the maximum size if the type is [`MemorySlice::Reserved`] or `0`
    /// if the type is [`MemorySlice::Free`].
    fn occupied_space(&self) -> u16 {
        use MemorySlice::*;
        match self {
            Reserved => SLICE_SIZE_U16,
            Free => 0_u16,
            KernelSpace(s)|UserLand(s) => SLICE_SIZE_U16 - *s,
        }
    }

    /// Increases the free space of the slice by the given `size`
    ///
    /// ## Returns
    ///
    /// Returns an [`Err`] if the slice is of type [`MemorySlice::Reserved`]
    /// or [`MemorySlice::Free`], or if the result of the addition exceeds
    /// the maximum slice size. Otherwise returns an empty [`Ok`].
    fn free_space(&mut self, size:u16) -> Result<(),MemoryError> {
        use MemorySlice::*;
        match self {
            Reserved => Err(MemoryError::ReservedSpace),
            Free => Err(MemoryError::ReleasingFreeSpace),
            KernelSpace(s)|UserLand(s) => {
                let new_size = *s + size;
                if new_size > SLICE_SIZE_U16 {
                    return Err(MemoryError::SpaceIssue);
                }
                Ok(*s = new_size)
            },
        }
    }

    /// Frees the whole space of the slice
    ///
    /// ## Returns
    ///
    /// Returns an [`Err`] if the slice is of type [`MemorySlice::Reserved`]
    /// or [`MemorySlice::Free`], otherwise returns an empty [`Ok`].
    fn free_all_space(&mut self) -> Result<(),MemoryError> {
        use MemorySlice::*;
        match self {
            Reserved => Err(MemoryError::ReservedSpace),
            Free => Err(MemoryError::ReleasingFreeSpace),
            KernelSpace(s)|UserLand(s) => Ok(*s = SLICE_SIZE_U16),
        }
    }

    /// Decreases the free space of the slice by the given `size`
    ///
    /// ## Returns
    ///
    /// Returns an [`Err`] if the slice is of type [`MemorySlice::Reserved`]
    /// or [`MemorySlice::Free`], or if `size` is bigger than the available
    /// space in the slice. Otherwise returns an empty [`Ok`].
    fn take_space(&mut self, size:u16) -> Result<(),MemoryError> {
        use MemorySlice::*;
        match self {
            Reserved => Err(MemoryError::ReservedSpace),
            Free => Err(MemoryError::BringingFreeSpace),
            KernelSpace(s)|UserLand(s) => {
                if size > *s {
                    return Err(MemoryError::SpaceIssue);
                }
                Ok(*s -= size)
            },
        }
    }

    /// Takes the whole space of the slice
    ///
    /// ## Returns
    ///
    /// Returns an [`Err`] if the slice is of type [`MemorySlice::Reserved`]
    /// or [`MemorySlice::Free`], otherwise returns an empty [`Ok`].
    fn take_all_space(&mut self) -> Result<(),MemoryError> {
        use MemorySlice::*;
        match self {
            Reserved => Err(MemoryError::ReservedSpace),
            Free => Err(MemoryError::BringingFreeSpace),
            KernelSpace(s)|UserLand(s) => Ok(*s = 0_u16),
        }
    }
}

impl Default for MemorySlice {
    /// Creates a memory slice of type [`MemorySlice::Free`]
    fn default() -> Self {
        MemorySlice::Free
    }
}

impl From<MemoryOwner> for MemorySlice {
    /// Creates a [`MemorySlice`] of the type defined by [`MemoryOwner`]
    fn from(owner:MemoryOwner) -> Self {
        match owner {
            MemoryOwner::Kernel => MemorySlice::KernelSpace(SLICE_SIZE_U16),
            MemoryOwner::User   => MemorySlice::UserLand(SLICE_SIZE_U16),
        }
    }
}

impl PartialEq<MemoryOwner> for MemorySlice {
    /// Checks whether a [`MemorySlice`] is of a type which is
    /// compatible with the given [`MemoryOwner`]
    fn eq(&self, owner:&MemoryOwner) -> bool {
        match (self,owner) {
            (MemorySlice::KernelSpace(_),MemoryOwner::Kernel) => true,
            (MemorySlice::UserLand(_),MemoryOwner::User) => true,
            _ => false,
        }
    }
}


/// A high level summary of the memory
///
/// Virtually splits the memory into slices of `4 KiB` each (the minimum
/// size of a page) and keeps track of whether a slice is reserved, free,
/// owned by a process or by the kernel.
struct MemoryMap {
    /// A sequence of memory slices representing portions of the memory.
    /// A slice can either be reserved, free or occupied by a page.
    slices : *mut MemorySlice,
    /// The total number of slices
    size : u64,
}

impl MemoryMap {
    /// Creates a new, un-initialized [`MemoryMap`]
    const
    fn new() -> Self {
        Self {
            slices : core::ptr::null_mut(),
            size   : 0_u64,
        }
    }

    /// Initializes the memory slices in the map
    ///
    /// `slices_paddr` represents the location in memory where the array
    /// of [`MemorySlice`] will be stored.
    /// `mappable_size` represents the total size of memory (in Bytes)
    /// that can be mapped in slices having a size of `4 KiB`.
    ///
    /// ## Panics
    ///
    /// Panics if the slices are already initialized
    fn init(&mut self, slices_paddr:PhysicalAddress, mappable_size:u64) {
        self.size = mappable_size / SLICE_SIZE_U64;
        if !self.slices.is_null() {
            crate::panic("Initializing non-null MemoryMap");
        }
        unsafe {
            self.slices = memset_defaulted::<MemorySlice>(slices_paddr.get(), self.size);
            let slices_size = core::mem::size_of::<MemorySlice>() as u64 * self.size;
            LAST_RESERVED_SLICE += slices_size / SLICE_SIZE_U64;
            for idx in 0..=LAST_RESERVED_SLICE as usize {
                *memory_map.slices.add(idx) = MemorySlice::Reserved;
            }
        }
    }
}


/// A lookup over a portion of the memory
struct MemoryLookup {
    /// A pointer to the beginning of the array of [`MemorySlice`]
    /// stored in the [`MemoryMap`]
    slices : *mut MemorySlice,
    /// The maximum slice index reachable through this lookup
    max : u64,
    /// The index of the slice that is currently being looked-up
    idx : u64,
}

impl MemoryLookup {
    /// Creates a new [`MemoryLookup`] iterator which starts from the slice
    /// that represents the given [`PhysicalAddress`] and ends at the last
    /// slice of the [`MemoryMap`]
    ///
    /// ## Returns
    ///
    /// Returns an [`Err`] if `paddr` is not aligned with the boundaries of a
    /// `4 KiB` page or if the first slice in the lookup range will be
    /// out-of-bounds, otherwise returns an [`Ok`] with the iterator.
    fn from(paddr:PhysicalAddress) -> Result<Self,MemoryError> {
        if !paddr.is_aligned(PageType::FourKiB) {
            return Err(MemoryError::UnalignedAddress);
        }
        let start = paddr.get() / SLICE_SIZE_U64;
        if unsafe { start >= memory_map.size } {
            return Err(MemoryError::InvalidRequest);
        }
        Ok(Self {
            slices : unsafe { memory_map.slices },
            max    : unsafe { memory_map.size },
            idx    : start,
        })
    }

    /// Creates a new [`MemoryLookup`] iterator which starts from the slice
    /// that represents the given [`PhysicalAddress`] and ends after a number
    /// of slices which cumulative size equals the size implicitly defined
    /// by the given [`PageType`] times the given number of pages
    ///
    /// ## Returns
    ///
    /// Returns an [`Err`] if `paddr` is not aligned with the boundaries
    /// defined by `ptype` or if the last slice in the lookup range will be
    /// out-of-bounds, otherwise returns an [`Ok`] with the iterator.
    fn range(paddr:PhysicalAddress, ptype:PageType, n:u64) -> Result<Self,MemoryError> {
        if !paddr.is_aligned(ptype) {
            return Err(MemoryError::UnalignedAddress);
        }
        let page_size : u64 = ptype.into();
        let n_slices = page_size / SLICE_SIZE_U64 * n;
        let start = paddr.get() / SLICE_SIZE_U64;
        let stop = start + n_slices;
        if unsafe { stop >= memory_map.size } {
            return Err(MemoryError::InvalidRequest);
        }
        Ok(Self {
            slices : unsafe { memory_map.slices },
            max    : stop,
            idx    : start,
        })
    }
}

impl Default for MemoryLookup {
    /// Creates a new [`MemoryLookup`] iterator starting from the first
    /// non-reserved slice
    fn default() -> Self {
        unsafe {
            Self {
                slices : memory_map.slices,
                max    : memory_map.size,
                idx    : LAST_RESERVED_SLICE + 1,
            }
        }
    }
}

impl Iterator for MemoryLookup {
    type Item = (PhysicalAddress, *mut MemorySlice);

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.max {
            let addr = self.idx * SLICE_SIZE_U64;
            let ret = unsafe {
                (PhysicalAddress::from(addr), self.slices.add(self.idx as usize))
            };
            self.idx += 1;
            return Some(ret);
        }
        None
    }
}


/// Initializes the [`MemoryMap`]
///
/// `memory_size` represents the total size of memory in Bytes
pub(super)
fn init(memory_size:u64) {
    crate::tty::print("[MEMORY]> Initializing memory map\n");

    let mappable_size = memory_size - (memory_size % SLICE_SIZE_U64);
    let slices_paddr = PhysicalAddress::from(KERNEL_PADDR_LIMIT+1);
    unsafe {
        memory_map.init(slices_paddr, mappable_size);
    }
}


/// Marks all the slices in the range `[beg_paddr,end_paddr)` as reserved
///
/// ## Panics
///
/// Panics if either `beg_paddr` or `end_paddr` are not aligned with 4096
/// boundaries
pub(in crate::memory)
fn set_reserved(
    beg_paddr:PhysicalAddress,
    end_paddr:PhysicalAddress,
) {
    if !beg_paddr.is_aligned(PageType::FourKiB) | !end_paddr.is_aligned(PageType::FourKiB) {
        crate::panic("Reserved address is not aligned");
    }
    let mut idx = (beg_paddr.get() / SLICE_SIZE_U64) as usize;
    let mut last_idx = (end_paddr.get() / SLICE_SIZE_U64) as usize;
    let max_idx = unsafe { memory_map.size } as usize;
    if last_idx > max_idx {
        last_idx = max_idx;
    }
    while idx < last_idx {
        let memslice = unsafe { &mut*memory_map.slices.add(idx) };
        if !memslice.is_reserved() {
            *memslice = MemorySlice::Reserved;
        }
        idx += 1;
    }
}


/// Marks as owned by `owner` all the memory slices starting from
/// `paddr` and up to the size implicitly defined by `page_type`
///
/// ## Returns
///
/// Returns an [`Err`] if `paddr` is not aligned with the boundaries
/// required by `page_type` or if any slice of memory in the range
/// is not free to take, otherwise returns an empty [`Ok`].
pub(in crate::memory)
fn acquire(
    page_paddr:PhysicalAddress,
    page_type:PageType,
    owner:MemoryOwner,
) -> Result<(),MemoryError> {
    acquire_range(page_paddr, page_type, 1, owner)
}

/// Marks as owned by `owner` all the memory slices starting from
/// `paddr` and up to the size implicitly defined by `page_type`
/// times `n_pages`
///
/// ## Returns
///
/// Returns an [`Err`] if `paddr` is not aligned with the boundaries
/// required by `page_type` or if any slice of memory in the range
/// is not free to take, otherwise returns an empty [`Ok`].
pub(in crate::memory)
fn acquire_range(
    page_paddr:PhysicalAddress,
    page_type:PageType,
    n_pages:u64,
    owner:MemoryOwner,
) -> Result<(),MemoryError> {
    for (_,slice) in MemoryLookup::range(page_paddr, page_type, n_pages)? {
        let memslice = unsafe { &mut *slice };
        if memslice.is_free() {
            *memslice = MemorySlice::from(owner);
        } else if memslice.is_reserved() {
            return Err(MemoryError::ReservedMemory);
        } else {
            return Err(MemoryError::TakingNonFreeMemory);
        }
    }
    Ok(())
}


/// Releases the ownership of all the memory slices starting from `paddr`
/// and up to the size implicitly defined by `page_type`
///
/// ## Warning
///
/// This function assumes that the given physical address actually points
/// to the base address of a page. There are no checks in place to verify
/// that.
///
/// ## Returns
///
/// Returns an [`Err`] if `paddr` is not aligned with the boundaries
/// required by `page_type` or if `owner` if different than the actual
/// owner of the memory, otherwise returns an empty [`Ok`]
pub(in crate::memory)
fn release(
    page_paddr:PhysicalAddress,
    page_type:PageType,
    owner:MemoryOwner,
) -> Result<(),MemoryError> {
    release_range(page_paddr, page_type, 1, owner)
}

/// Releases the ownership of all the memory slices starting from `paddr`
/// and up to the size implicitly defined by `page_type` times `n_pages`
///
/// ## Warning
///
/// This function assumes that the given physical address actually points
/// to the base address of a page. There are no checks in place to verify
/// that.
///
/// ## Returns
///
/// Returns an [`Err`] if `paddr` is not aligned with the boundaries
/// required by `page_type` or if `owner` if different than the actual
/// owner of the memory, otherwise returns an empty [`Ok`]
pub(in crate::memory)
fn release_range(
    page_paddr:PhysicalAddress,
    page_type:PageType,
    n_pages:u64,
    owner:MemoryOwner,
) -> Result<(),MemoryError> {
    for (_,slice) in MemoryLookup::range(page_paddr, page_type, n_pages)? {
        let memslice = unsafe { &mut *slice };
        if *memslice != owner {
            return Err(MemoryError::OwnershipMismatch);
        } else if memslice.is_reserved() {
            return Err(MemoryError::ReservedMemory);
        }
        match memslice.is_free() {
            true  => return Err(MemoryError::DroppingFreeMemory),
            false => *memslice = MemorySlice::Free,
        }
    }
    Ok(())
}


/// Checks whether the given [`PhysicalAddress`] points to a location
/// in memory that has enough contiguous space to be able to host
/// a page of the given [`PageType`]
///
/// ## Returns
///
/// Returns an [`Err`] if `paddr` is not aligned with the boundaries
/// required by `page_type`, otherwise returns an [`Ok`] with the
/// result of the check.
pub(in crate::memory)
fn is_available(
    paddr:PhysicalAddress,
    page_type:PageType
) -> Result<bool,MemoryError> {
    is_available_range(paddr, page_type, 1)
}

/// Checks whether the given [`PhysicalAddress`] points to a location
/// in memory that has enough contiguous space to be able to host
/// a concatenation of `n_pages` of the given [`PageType`]
///
/// ## Returns
///
/// Returns an [`Err`] if `paddr` is not aligned with the boundaries
/// required by `page_type`, otherwise returns an [`Ok`] with the
/// result of the check.
pub(in crate::memory)
fn is_available_range(
    paddr:PhysicalAddress,
    page_type:PageType,
    n_pages:u64,
) -> Result<bool,MemoryError> {
    let count = MemoryLookup::range(paddr, page_type, n_pages)?
        .filter(|(_,s)| unsafe { (*(*s)).is_free() })
        .count() as u64;
    Ok(count >= n_pages)
}


/// Searches for a location in memory that has enough contiguous space
/// to be able to host a page of the given [`PageType`]
///
/// ## Returns
///
/// Returns a [`Some`] containing the PhysicalAddress of the memory
/// location if a suitable one is found, otherwise returns [`None`].
pub(in crate::memory)
fn find_available(page_type:PageType) -> Option<PhysicalAddress> {
    find_available_range(page_type, 1)
}

/// Searches for a location in memory that has enough contiguous space
/// to be able to host a concatenation of `n_pages` of the given [`PageType`]
///
/// ## Returns
///
/// Returns a [`Some`] containing the [`PhysicalAddress`] of the memory
/// location if a suitable one is found, otherwise returns [`None`].
pub(in crate::memory)
fn find_available_range(page_type:PageType, n_pages:u64) -> Option<PhysicalAddress> {
    let page_size : u64 = page_type.into();
    let requested_size = n_pages * page_size;
    let mut size = 0_u64;
    let mut address : Option<PhysicalAddress> = None;
    for (paddr,slice) in MemoryLookup::default() {
        if unsafe { (*slice).is_free() } {
            match address {
                None => if paddr.is_aligned(page_type) {
                    address = Some(paddr);
                    if requested_size <= SLICE_SIZE_U64 {
                        return address;
                    }
                    size = SLICE_SIZE_U64;
                },
                Some(_) => {
                    size += SLICE_SIZE_U64;
                    if size >= requested_size {
                        return address;
                    }
                },
            }
        } else {
            size = 0;
            address = None;
        }
    }
    None
}


/// Checks whether the memory location pointed to by `paddr` has at least
/// as much free space as `space_size`
///
/// ## Warning
///
/// This function assumes that the given [`PhysicalAddress`] actually points
/// to the base address of a page.
///
/// ## Returns
///
/// Returns an [`Err`] if `paddr` is not aligned with the boundaries
/// required by `page_type` or if `owner` if different than the actual
/// owner of the memory, otherwise returns an [`Ok`] with the result
/// of the check.
pub(in crate::memory)
fn has_space(
    paddr:PhysicalAddress,
    page_type:PageType,
    space_size:u64,
    owner:MemoryOwner,
) -> Result<bool,MemoryError> {
    has_space_range(paddr, page_type, 1, space_size, owner)
}

/// Checks whether the memory location pointed to by `paddr` has at least
/// as much free space as `space_size`
///
/// ## Warning
///
/// This function assumes that the given [`PhysicalAddress`] actually points
/// to the base address of a page.
///
/// ## Returns
///
/// Returns an [`Err`] if `paddr` is not aligned with the boundaries
/// required by `page_type` or if `owner` is different than the actual
/// owner of the memory, otherwise returns an [`Ok`] with the result
/// of the check.
pub(in crate::memory)
fn has_space_range(
    paddr:PhysicalAddress,
    page_type:PageType,
    n_pages:u64,
    space_size:u64,
    owner:MemoryOwner,
) -> Result<bool,MemoryError> {
    let mut found = false;
    let mut avl_size = 0_u64;
    for (_,slice) in MemoryLookup::range(paddr, page_type, n_pages)? {
        let memslice = unsafe { & *slice };
        if memslice != &owner {
            return Err(MemoryError::OwnershipMismatch);
        } else if memslice.is_reserved() {
            return Err(MemoryError::ReservedMemory);
        } else {
            found |= true;
        }
        if avl_size == 0 {
            avl_size += memslice.available_space() as u64;
        } else {
            let slice_space = memslice.available_space() as u64;
            avl_size += slice_space;
            if slice_space != SLICE_SIZE_U64 {
                break;
            }
        }
    }
    match found {
        true  => Ok(avl_size >= space_size),
        false => Err(MemoryError::NotFound),
    }
}


/// Returns the total free space of the memory location pointed to by
/// the given [`PhysicalAddress`]
///
/// ## Warning
///
/// This function assumes that the given [`PhysicalAddress`] actually points
/// to the base address of a page.
///
/// ## Returns
///
/// Returns an [`Err`] if `paddr` is not aligned with the boundaries
/// required by `page_type` or if `owner` if different than the actual
/// owner of the memory, otherwise returns an [`Ok`] with the result.
pub(in crate::memory)
fn get_space(
    paddr:PhysicalAddress,
    page_type:PageType,
    owner:MemoryOwner,
) -> Result<u64,MemoryError> {
    get_space_range(paddr, page_type, 1, owner)
}

/// Returns the total free space of the memory location pointed to by
/// the given [`PhysicalAddress`]
///
/// ## Warning
///
/// This function assumes that the given [`PhysicalAddress`] actually points
/// to the base address of a page
///
/// ## Returns
///
/// Returns an [`Err`] if `paddr` is not aligned with the boundaries
/// required by `page_type` or if `owner` if different than the actual
/// owner of the memory, otherwise returns an [`Ok`] with the result
/// of the check.
pub(in crate::memory)
fn get_space_range(
    paddr:PhysicalAddress,
    page_type:PageType,
    n_pages:u64,
    owner:MemoryOwner,
) -> Result<u64,MemoryError> {
    let mut found = false;
    let mut avl_size = 0_u64;
    for (_,slice) in MemoryLookup::range(paddr, page_type, n_pages)? {
        let memslice = unsafe { & *slice };
        if memslice != &owner {
            return Err(MemoryError::OwnershipMismatch);
        } else if memslice.is_reserved() {
            return Err(MemoryError::ReservedMemory);
        }
        found |= true;
        avl_size += memslice.available_space() as u64;
    }
    if found {
        return Ok(avl_size);
    }
    Err(MemoryError::NotFound)
}


/// Takes the available space of all the slices starting from `paddr`
/// until the whole size defined by `space_size` is taken
///
/// ## Warning
///
/// This function assumes that the space in the first slice is actually at the
/// end of the slice and that the space in the last slice is actually at the
/// beginning of the slice.
///
/// ## Returns
///
/// Returns an [`Err`] if the space in the slices is not contiguous after the
/// first slice and up to the last (namely, all the slices except the first
/// and the last must have the whole slice space available), if the `owner`
/// is different than the memory owner of any of the slices, or if any of the
/// slices is marked as reserved or free. Otherwise returns an empty [`Ok`].
pub(in crate::memory)
fn take_space_unconstrained(
    mut paddr:PhysicalAddress,
    mut alloc_space:u64,
    owner:MemoryOwner,
) -> Result<(),MemoryError> {
    // NOTE:
    //   Might be that not all the space of the first slice has to be taken.
    //   If the total size to take is 1024 B, and the available size in the
    //   slice is 1024 B over 4096 B total, but the layout in memory is
    //   2048 B taken + 512 B free + 1024 B taken + 512 B free, than only the
    //   last 512 B have to be marked as taken.
    let first_slice_idx = paddr.get() as usize / SLICE_SIZE_USIZE;
    let first_slice = unsafe { &mut *memory_map.slices.add(first_slice_idx) };
    let first_slice_space = first_slice.available_space() as u64;
    if first_slice_space >= alloc_space {
        return first_slice.take_space(alloc_space as u16);
    } else {
        first_slice.take_space(first_slice_space as u16)?;
        paddr.force_align_to_upper(PageType::FourKiB);
        alloc_space -= first_slice_space;
    }

    use core::cmp::Ordering::*;
    for (_,slice) in MemoryLookup::from(paddr)? {
        let memslice = unsafe { &mut *slice };
        if *memslice != owner {
            return Err(MemoryError::OwnershipMismatch);
        }
        let slice_space = memslice.available_space() as u64;
        match slice_space.partial_cmp(&alloc_space) {
            None => return Err(MemoryError::InvalidRequest),
            Some(Equal) => return memslice.take_all_space(),
            Some(Greater) => return memslice.take_space(alloc_space as u16),
            Some(Less) => match slice_space == SLICE_SIZE_U64 {
                false => return Err(MemoryError::SpaceIssue),
                true  => memslice.take_all_space()?,
            },
        }
        alloc_space -= slice_space as u64;
    }
    if alloc_space > 0 {
        return Err(MemoryError::SpaceIssue);
    }
    Ok(())
}


/// Drops the occupied space of all the slices starting from `paddr`
/// until the whole size defined by `space_size` is freed
///
/// ## Warning
///
/// This function assumes that the space in the first slice is actually at the
/// end of the slice and that the space in the last slice is actually at the
/// beginning of the slice.
///
/// ## Returns
///
/// Returns an [`Err`] if the space in the slices is not contiguous after the
/// first slice and up to the last (namely, all the slices except the first
/// and the last must have the whole slice space occupied), if the `owner` is
/// different than the memory owner of any of the slices, or if any of the
/// slices is marked as reserved or free. Otherwise returns an empty [`Ok`].
pub(in crate::memory)
fn drop_space_unconstrained(
    mut paddr:PhysicalAddress,
    mut dealloc_space:u64,
    owner:MemoryOwner,
) -> Result<(),MemoryError> {
    // NOTE:
    //   Might be that not all the space of the first slice has to be freed.
    //   If the total size to take is 1024 B, and the occupied size in the
    //   slice is 1024 B over 4096 B total, but the layout in memory is
    //   2048 B free + 512 B taken + 1024 B free + 512 B taken, than only the
    //   last 512 B have to be marked as freed.
    let first_slice_idx = paddr.get() as usize / SLICE_SIZE_USIZE;
    let first_slice = unsafe { &mut *memory_map.slices.add(first_slice_idx) };
    let first_slice_space = first_slice.occupied_space() as u64;
    if first_slice_space >= dealloc_space {
        return first_slice.free_space(dealloc_space as u16);
    } else {
        first_slice.free_space(first_slice_space as u16)?;
        paddr.force_align_to_upper(PageType::FourKiB);
        dealloc_space -= first_slice_space;
    }

    use core::cmp::Ordering::*;
    for (_,slice) in MemoryLookup::from(paddr)? {
        let memslice = unsafe { &mut *slice };
        if *memslice != owner {
            return Err(MemoryError::OwnershipMismatch);
        }
        let slice_space = memslice.occupied_space() as u64;
        match slice_space.partial_cmp(&dealloc_space) {
            None => return Err(MemoryError::InvalidRequest),
            Some(Equal) => return memslice.free_all_space(),
            Some(Greater) => return memslice.free_space(dealloc_space as u16),
            Some(Less) => match slice_space == 0_u64 {
                true  => return Err(MemoryError::SpaceIssue),
                false => memslice.free_all_space()?,
            },
        }
        dealloc_space -= slice_space as u64;
    }
    if dealloc_space > 0 {
        return Err(MemoryError::SpaceIssue);
    }
    Ok(())
}
