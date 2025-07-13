use crate::OkOrPanic;
use crate::memory::*;
use crate::memory::address::*;
use crate::memory::paging::*;
use crate::memory::paging::tracing::{
    TracingPageError, MetadataIterator, MetadataIteratorMut
};
use crate::memory::paging::tracing::management::TRACE_PAGE_SIZE;

use core::ops::{AddAssign, SubAssign, Sub, ShlAssign, ShrAssign};


/// Represents the size in memory of a [`TracingPage`]
pub(in crate::memory::paging::tracing) const
TRACING_PAGE_SIZE   : usize = TRACE_PAGE_SIZE as usize;

/// Represents the number of entries of a [`Metadata`] array
pub(in crate::memory::paging::tracing) const
METADATA_ARRAY_SIZE : usize = (TRACING_PAGE_SIZE - core::mem::size_of::<usize>()) / core::mem::size_of::<Metadata>();


/// Holds some [`Metadata`] that need to be moved to the next page
///
/// Follows the principles of an ordered queue: elements are popped from the
/// front and pushed to the back, and shall be inserted in another array in
/// the same order they already are.
///
/// ## Warning
///
/// Care must be taken when pushing and popping elements, in order to respect
/// the order of the entries.
///
/// ## Example
///
/// ```
/// let metadata1 = Metadata::new_free(0x1000.into(), 0x1000.into(), 0x1000);
/// let metadata2 = Metadata::new_free(0x2000.into(), 0x2000.into(), 0x2000);
/// let mut excess = Excess::none();
/// excess.push(metadata1);
/// assert_eq!(excess, Excess::single(metadata1));
/// excess.push(metadata2);
/// assert_eq!(excess, Excess::double(metadata1, metadata2));
/// assert_eq!(excess.pop(), Some(metadata1));
/// assert_eq!(excess, Excess::single(metadata2));
/// assert_eq!(excess.pop(), Some(metadata2);
/// assert_eq!(excess, Excess::none());
/// assert_eq!(excess.pop(), None);
/// ```
pub(in crate::memory::paging::tracing)
struct Excess {
    size : usize,
    data : [Metadata; 2],
}

impl Excess {
    /// Returns an empty [`Excess`]
    fn none() -> Self {
        Self {
            size : 0,
            data : [Metadata::default(); 2],
        }
    }

    /// Returns an [`Excess`] containing the given element
    fn single(md:Metadata) -> Self {
        Self {
            size : 1,
            data : [md, Metadata::default()],
        }
    }

    /// Returns an [`Excess`] containing the given elements, in the same order
    fn double(md1:Metadata, md2:Metadata) -> Self {
        Self {
            size : 2,
            data : [md1, md2],
        }
    }

    /// Checks whether the [`Excess`] is empty
    pub(in crate::memory::paging::tracing)
    fn is_none(&self) -> bool {
        self.size == 0
    }

    /// Returns the number of elements stored
    pub(in crate::memory::paging::tracing)
    fn size(&self) -> usize {
        self.size
    }

    /// Pushes an element on the back side
    fn push(&mut self, md:Metadata) {
        if self.size == 2 {
            crate::panic("Excess size exceeded");
        }
        self.data[self.size] = md;
        self.size += 1;
    }

    /// Pushes an element on the back side, without checking
    ///
    /// ## Panics
    ///
    /// Panics if the [`Excess`] already contains two elements
    fn push_unchecked(&mut self, md:Metadata) {
        self.data[self.size] = md;
        self.size += 1;
    }

    /// Pops an element from the front side
    pub(in crate::memory::paging::tracing)
    fn pop(&mut self) -> Option<Metadata> {
        if self.size == 0 {
            return None;
        }
        Some(self.pop_unchecked())
    }

    /// Pops an element from the front side, without checking
    ///
    /// ## Warning
    ///
    /// If the [`Excess`] is empty, the returned [`Metadata`] will be
    /// of none type
    fn pop_unchecked(&mut self) -> Metadata {
        let second = core::mem::take(&mut self.data[1]);
        let first = core::mem::replace(&mut self.data[0], second);
        self.size -= 1;
        first
    }
}

impl Default for Excess {
    fn default() -> Self {
        Self::none()
    }
}

impl From<Option<Metadata>> for Excess {
    fn from(md_opt:Option<Metadata>) -> Self {
        match md_opt {
            Some(md) => Self::single(md),
            None => Self::none(),
        }
    }
}


/// Holds some [`Metadata`] that need to be somehow inserted in or removed from
/// the following page
pub(in crate::memory::paging::tracing)
enum Reminder {
    /// This [`Metadata`] needs to be summed to the first entry of the next page.
    /// In case both the entries are marked as free, they shall be merged if they
    /// are contiguous. In case both the entries are taken, or they are of a
    /// different type, this [`Metadata`] shall be inserted as first entry and all
    /// the other entries shall be shifted by one position.
    Positive(Metadata),
    /// This [`Metadata`] needs to be subtracted from the first entry of the next page.
    /// This is only possible if the other entry is marked as free, otherwise it shall
    /// be considered an error (a bad one).
    Negative(Metadata),
    /// Nothing to do
    Zero,
}

impl Reminder {
    /// Returns a [`Reminder::Zero`]
    pub(in crate::memory::paging::tracing)
    fn zero() -> Self {
        Self::Zero
    }

    /// Returns a [`Reminder::Positive`] containing the given [`Metadata`]
    ///
    /// If the [`Metadata`] has no size, a [`Reminder::Zero`] is returned.
    pub(in crate::memory::paging::tracing)
    fn positive(md:Metadata) -> Self {
        match md.info.size() {
            0 => Self::Zero,
            _ => Self::Positive(md),
        }
    }

    /// Returns a [`Reminder::Negative`] containing the given [`Metadata`]
    ///
    /// If the [`Metadata`] has no size, a [`Reminder::Zero`] is returned.
    pub(in crate::memory::paging::tracing)
    fn negative(md:Metadata) -> Self {
        match md.info.size() {
            0 => Self::Zero,
            _ => Self::Negative(md),
        }
    }

    /// Checks whether the reminder is a [`Reminder::Zero`] or not
    pub(in crate::memory::paging::tracing)
    fn is_zero(&self) -> bool {
        match self {
            Self::Zero => true,
            _ => false,
        }
    }
}


/// Used to indicate from wich side an operation should be executed
///
/// ## Example
///
/// ```
/// let paddr = PhysicalAddress::from(0x10000)
/// let laddr = LogicalAddress::from(0x10000)
/// let size = 0x1000;
/// let mut md1 = Metadata::new_free(paddr, laddr, size);
/// let mut md2 = Metadata::new_free(paddr, laddr, size);
/// md1 -= (0x100, Side::Left);
/// md2 -= (0x100, Side::Right);
/// assert_eq!(md1.size(), 0xF00);
/// assert_eq!(md2.size(), 0xF00);
/// assert_eq!(md1.paddr, 0x10100);
/// assert_eq!(md2.paddr, 0x10000);
/// ```
enum Side {
    Left,
    Right,
}


/// Represents a portion of contiguous memory of any kind
///
/// Can store a maximum size of 18446744073709551615 (0xFFFFFFFFFFFFFFFF) Bytes,
/// equivalent to 17'179'869'184 GiB
///
/// ## Usage
///
/// Contiguous `Free` elements shall be merged toghether (if they're strictly contiguous
/// *both physically and logically*) in order to save space in the array, and also because
/// there's no reason to keep them separate.
/// Elements that are `Taken` shall *never* be merged toghether, since each element of this
/// type represents a single object allocated in memory and merging them will cause havoc.
#[derive(Clone,Copy)]
#[repr(align(8))]
pub(in crate::memory::paging::tracing)
enum Info {
    /// Represents a portion of memory with something allocated in it
    Taken(u64),
    /// Represents a portion of memory that is free for allocation
    Free(u64),
    /// Placeholder for an uninitialized [`Metadata`] entry, with a null address and no size
    None,
}

impl Info {
    /// Tells whether an entry is not initialized
    pub(in crate::memory::paging::tracing)
    fn is_none(&self) -> bool {
        match self {
            Info::None => true,
            _ => false,
        }
    }

    /// Tells whether an entry is marked as free
    fn is_free(&self) -> bool {
        match self {
            Info::Free(_) => true,
            _ => false,
        }
    }

    /// Marks the entry as free
    fn set_free(&mut self) {
        match self {
            Info::Taken(s) => *self = Info::Free(*s),
            _ => (),
        }
    }

    /// Tells whether an entry is marked as taken
    fn is_taken(&self) -> bool {
        match self {
            Info::Taken(_) => true,
            _ => false,
        }
    }

    /// Marks the entry as taken
    fn set_taken(&mut self) {
        match self {
            Info::Free(s) => *self = Info::Taken(*s),
            _ => (),
        }
    }

    /// Returns the size of an entry (if any)
    fn size(&self) -> u64 {
        match self {
            Info::Free(s)|Info::Taken(s) => *s,
            _ => 0_u64,
        }
    }

    /// Sets the given size to the entry
    ///
    /// ## Returns
    ///
    /// Returns an [`Err`] if the entry is of none type, otherwise returns
    /// an empty [`Ok`].
    fn set_size(&mut self, size:u64) -> Result<(),TracingPageError> {
        match self {
            Info::Free(s)|Info::Taken(s) => Ok(*s = size),
            Info::None => Err(TracingPageError::EntryIsNone),
        }
    }
}

impl PartialEq<u64> for Info {
    /// Checks for partial equality on the entry
    ///
    /// ## Returns
    ///
    /// Returns `true` if the given `size` equals the size of the entry, regardless
    /// of its type. Otherwise returns `false`.
    fn eq(&self, size:&u64) -> bool {
        match self {
            Info::Free(s)|Info::Taken(s) => s == size,
            _ => false,
        }
    }
}

impl PartialOrd<u64> for Info {
    /// Performs a partial comparison between the entry and the given `size`
    ///
    /// ## Returns
    ///
    /// Returns [`None`] if the entry is of [`None`] type, otherwise returns the
    /// result of partial comparison between the given `size` and the entry's
    /// size, regardless of its type.
    fn partial_cmp(&self, size:&u64) -> Option<core::cmp::Ordering> {
        match self {
            Info::Free(s)|Info::Taken(s) => if s == size {
                Some(core::cmp::Ordering::Equal)
            } else if s < size {
                Some(core::cmp::Ordering::Less)
            } else {
                Some(core::cmp::Ordering::Greater)
            },
            _ => None,
        }
    }
}

impl AddAssign<u64> for Info {
    /// Increases the size of the entry by the given value
    ///
    /// ## Warning
    ///
    /// This function doesn't check whether the final size will exceed [`u64::MAX`]
    fn add_assign(&mut self, rhs:u64) {
        match self {
            Info::Free(s)|Info::Taken(s) => *s += rhs,
            _ => (),
        }
    }
}

impl SubAssign<u64> for Info {
    /// Reduces the size of the entry by the given value
    ///
    /// ## Warning
    ///
    /// This function doesn't check whether the final size will be below [`u64::MIN`]
    fn sub_assign(&mut self, rhs:u64) {
        match self {
            Info::Free(s)|Info::Taken(s) => *s -= rhs,
            _ => (),
        }
    }
}

impl Sub<u64> for Info {
    type Output = Self;
    /// Reduces the size of the entry by the given value and returns it
    ///
    /// ## Warning
    ///
    /// This function doesn't check whether the final size will be below [`u64::MIN`]
    fn sub(self, rhs:u64) -> Self::Output {
        match self {
            Info::Free(s) => Info::Free(s - rhs),
            Info::Taken(s) => Info::Taken(s - rhs),
            _ => self,
        }
    }
}


/// Stores information about a slice of dynamic memory
#[derive(Clone,Copy)]
#[repr(align(8))]
pub(in crate::memory::paging::tracing)
struct Metadata {
    /// The physical address at which the memory slice begins
    paddr : PhysicalAddress,
    /// The logical address at which the memory slice begins
    laddr : LogicalAddress,
    /// The information about the size and type of the slice
    info : Info,
}

impl Metadata {
    /// Creates a new entry with the given data and marks it as free
    pub(in crate::memory::paging::tracing)
    fn new_free(pa:PhysicalAddress, la:LogicalAddress, sz:u64) -> Self {
        Self {
            paddr : pa,
            laddr : la,
            info  : Info::Free(sz)
        }
    }

    /// Creates a new entry with the given data and marks it as taken
    pub(in crate::memory::paging::tracing)
    fn new_taken(pa:PhysicalAddress, la:LogicalAddress, sz:u64) -> Self {
        Self {
            paddr : pa,
            laddr : la,
            info  : Info::Taken(sz)
        }
    }

    /// Returns the size of the entry
    pub(in crate::memory::paging::tracing)
    fn size(&self) -> u64 {
        self.info.size()
    }

    /// Replaces the size of an entry with the given size
    ///
    /// ## Warning
    ///
    /// This function does not update the addresses, use with great care.
    fn set_size(&mut self, new_size:u64) -> Result<(),TracingPageError> {
        self.info.set_size(new_size)
    }

    /// Tells whether an entry is not initialized
    pub(in crate::memory::paging::tracing)
    fn is_none(&self) -> bool {
        self.info.is_none()
    }

    /// Tells whether an entry is marked as free
    pub(in crate::memory::paging::tracing)
    fn is_free(&self) -> bool {
        self.info.is_free()
    }

    /// Marks the entry as free
    ///
    /// Has no effect if the entry is not marked as taken
    pub(in crate::memory::paging::tracing)
    fn set_free(&mut self) {
        self.info.set_free();
    }

    /// Tells whether an entry is marked as taken
    pub(in crate::memory::paging::tracing)
    fn is_taken(&self) -> bool {
        self.info.is_taken()
    }

    /// Marks the entry as taken
    ///
    /// Has no effect if the entry is not marked as free
    pub(in crate::memory::paging::tracing)
    fn set_taken(&mut self) {
        self.info.set_taken();
    }

    /// Returns the [`PhysicalAddress`] at which the entry starts
    pub(in crate::memory::paging::tracing)
    fn lower_paddr(&self) -> PhysicalAddress {
        self.paddr
    }

    /// Returns the [`PhysicalAddress`] that is `1 B` off the end of the entry
    pub(in crate::memory::paging::tracing)
    fn higher_paddr(&self) -> PhysicalAddress {
        self.paddr + self.info.size()
    }

    /// Returns the [`LogicalAddress`] at which the entry starts
    pub(in crate::memory::paging::tracing)
    fn lower_laddr(&self) -> LogicalAddress {
        self.laddr
    }

    /// Returns the [`LogicalAddress`] that is `1 B` off the end of the entry
    pub(in crate::memory::paging::tracing)
    fn higher_laddr(&self) -> LogicalAddress {
        self.laddr + self.info.size()
    }

    /// Checks whether the give [`PhysicalAddress`] is contained in the entry
    ///
    /// ## Note
    ///
    /// An entry may refer to a large chunk of memory in case it groups the memory
    /// of many different pages that are contiguous (both physically and logically),
    /// so in case only some of these pages need to be removed, an entry may happen
    /// not to start with the exact address of the first of these pages
    fn contains(&self, paddr:PhysicalAddress) -> bool {
        (self.lower_paddr() <= paddr) & (paddr < self.higher_paddr())
    }

    /// Checks whether the beginning of the entry is contiguous to the end of another
    ///
    /// ## Returns
    ///
    /// Returns `true` if this entry is physically and logically contiguous to
    /// the other entry (starts at the end of it), otherwise returns `false`.
    fn is_contiguous_to(&self, other:&Metadata) -> bool {
        (self.lower_paddr() == other.higher_paddr()) & (self.lower_laddr() == other.higher_laddr())
    }
}

impl Default for Metadata {
    /// Creates an uninitialized entry of type none
    fn default() -> Self {
        Self {
            paddr : PhysicalAddress::from(0),
            laddr : LogicalAddress::from(0),
            info  : Info::None,
        }
    }
}

impl AddAssign<(u64,Side)> for Metadata {
    /// Increases the size and updates the addresses
    ///
    /// The `side` argument defines toward which side the entry should be expanded
    fn add_assign(&mut self, (size,side):(u64,Side)) {
        self.info += size; // size must increases regardless of the side
        match side {
            Side::Left => { // decrease the addresses to reach a lower memory location
                self.paddr -= size;
                self.laddr -= size;
            },
            Side::Right => (), // nothing to do, addresses remain unchanged
        }
    }
}

impl SubAssign<(u64,Side)> for Metadata {
    /// Decreases the size and updates the addresses
    ///
    /// The `side` argument defines toward which side the entry should be expanded
    fn sub_assign(&mut self, (size,side):(u64,Side)) {
        self.info -= size; // size must decreases regardless of the side
        match side {
            Side::Left => { // increase the addresses to reach a higher memory location
                self.paddr += size;
                self.laddr += size;
            },
            Side::Right => (), // nothing to do, addresses remain unchanged
        }
    }
}

impl Sub<(u64,Side)> for Metadata {
    type Output = Metadata;
    /// Returns the entry with decreased size and updated addresses
    ///
    /// The `side` argument defines toward which side the entry should be expanded
    fn sub(self, args:(u64,Side)) -> Self::Output {
        let mut md = self;
        md -= args;
        md
    }
}

impl ShlAssign<u64> for Metadata {
    /// Shifts the addresses towards the left by the given amount of Bytes
    fn shl_assign(&mut self, size:u64) {
        self.paddr -= size;
        self.laddr -= size;
    }
}

impl ShrAssign<u64> for Metadata {
    /// Shifts the addresses towards the right by the given amount of Bytes
    fn shr_assign(&mut self, size:u64) {
        self.paddr += size;
        self.laddr += size;
    }
}


/// Stores information about the dynamic memory associated to a single process
///
/// Each [`TracingPage`] has a size of `~2 MiB` and contains `65535` [`Metadata`] entries.
/// The array of [`Metadata`] entries is sorted by physical address, increasingly.
#[repr(align(8))]
pub(in crate::memory::paging)
struct TracingPage {
    /// The number of non-none entries in the array
    size : usize,
    /// The array of [`Metadata`] entries
    metadata : [Metadata; METADATA_ARRAY_SIZE],
}

impl TracingPage {
    /// Casts the [`PhysicalAddress`] stored in the [`Bitmap`] of the given
    /// [`PageTableEntry`] into a mutable pointer to a [`TracingPage`]
    ///
    /// ## Warning
    ///
    /// This function implies dereferencing both the entry and the memory
    /// location it points to
    pub(in crate::memory::paging::tracing)
    fn from_table_entry(entry:PageTableEntry) -> *mut TracingPage {
        let entry_paddr : PhysicalAddress = entry.bitmap().address(PageType::TwoMiB).into();
        Self::cast_mut(entry_paddr)
    }

    /// Returns an iterator over the [`Metadata`] array
    pub(in crate::memory::paging::tracing)
    fn iterate(&self) -> MetadataIterator {
        MetadataIterator::new(&self.metadata, 0)
    }

    /// Returns a mutable iterator over the [`Metadata`] array
    pub(in crate::memory::paging::tracing)
    fn iterate_mut(&mut self) -> MetadataIteratorMut {
        MetadataIteratorMut::new(&mut self.metadata, 0)
    }

    /// Returns the number of entries in the page
    pub(in crate::memory::paging::tracing)
    fn size(&self) -> usize {
        self.size
    }

    /// Returns the number of unused entries in the page
    fn unused_size(&self) -> usize {
        METADATA_ARRAY_SIZE - self.size
    }

    /// Checks whether the page contains any valid [`Metadata`] or not
    pub(in crate::memory::paging::tracing)
    fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Checks whether the array has reached its maximum size or not
    pub(in crate::memory::paging::tracing)
    fn is_full(&self) -> bool {
        self.size == METADATA_ARRAY_SIZE
    }

    /// Returns the index of the last valid entry of the array, or `0`
    /// if the array is empty
    fn last_index(&self) -> usize {
        match self.is_empty() {
            true  => self.size,
            false => self.size - 1,
        }
    }

    /// Checks whether the given index is the last index
    fn is_last_index(&self, idx:usize) -> bool {
        idx == self.last_index()
    }

    /// Returns the lower [`PhysicalAddress`] of the array
    ///
    /// To retrieve the lower [`PhysicalAddress`] of the array, [`Metadata::lower_paddr()`]
    /// is called on the entry at index `0`.
    ///
    /// ## Warning
    ///
    /// This function assumes the array is not empty, in which case a null
    /// [`PhysicalAddress`] will be returned
    pub(in crate::memory::paging::tracing)
    fn lower_paddr(&self) -> PhysicalAddress {
        self.metadata[0].lower_paddr()
    }

    /// Returns the higher [`PhysicalAddress`] of the array
    ///
    /// To retrieve the higher [`PhysicalAddress`] of the array, [`Metadata::higher_paddr()`]
    /// is called on the entry at the last index
    ///
    /// ## Warning
    ///
    /// This function assumes the array is not empty, in which case a null
    /// [`PhysicalAddress`] will be returned
    fn higher_paddr(&self) -> PhysicalAddress {
        self.metadata[self.last_index()].higher_paddr()
    }

    /// Checks whether the given [`PhysicalAddress`] is inside the range of addresses
    /// of the whole array
    ///
    /// To be inside the range of addresses of the array, the given address
    /// must be _greater or equal to the lower address_ in the array and
    /// _smaller or equal to the higher address_.
    pub(in crate::memory::paging::tracing)
    fn contains_paddr(&self, paddr:PhysicalAddress) -> bool {
        // IMPORTANT NOTE:
        //   Can't do the same with logical addresses, since the array is sorted
        //   by physical addresses and logical ones are not mapped 1:1 with them.
        //   It can even be that the logical address of the first entry is higher
        //   than the logical address of the last one.
        if self.is_empty() {
            return false;
        }
        (self.lower_paddr() <= paddr) & (paddr <= self.higher_paddr())
    }

    /// Checks whether the given [`PhysicalAddress`] is strictly inside in the
    /// range of addresses of the array
    ///
    /// To be inside the range of addresses of the array, the given address
    /// must be _greater or equal to the lower address_ in the array and
    /// _strictly smaller than the higher address_.
    pub(in crate::memory::paging::tracing)
    fn contains_paddr_strict(&self, paddr:PhysicalAddress) -> bool {
        // IMPORTANT NOTE:
        //   Can't do the same with logical addresses, since the array is sorted
        //   by physical addresses and logical ones are not mapped 1:1 with them.
        //   It can even be that the logical address of the first entry is higher
        //   than the logical address of the last one.
        if self.is_empty() {
            return false;
        }
        (self.lower_paddr() <= paddr) & (paddr < self.higher_paddr())
    }

    /// Copies all the [`Metadata`] entries starting from the given index and up to the
    /// end of the array, toward the left by the given number of positions
    ///
    /// ## Warning
    ///
    /// This function just copies data and does not shrink the array after shifting.
    ///
    /// ## Returns
    ///
    /// Returns an [`Err`] if `n_shifts` is greater than `src_idx` or `src_idx` is greater
    /// than or equal to the the size of the array. Otherwise returns an empty [`Ok`].
    fn shift_left(&mut self, src_idx:usize, n_shifts:usize) -> Result<(),TracingPageError> {
        if (src_idx < n_shifts) | (src_idx >= self.size) {
            return Err(TracingPageError::LeftShiftPreconditions);
        }
        let dst_idx = src_idx - n_shifts;
        let end_src_idx = src_idx + (self.size - src_idx);
        self.metadata.copy_within(src_idx..end_src_idx, dst_idx);
        Ok(())
    }

    /// Copies all the [`Metadata`] entries starting from the given index and up to the
    /// end of the array, toward the right by the given number of positions
    ///
    /// ## Warning
    ///
    /// This function also increases the size of the array accordingly.
    ///
    /// ## Returns
    ///
    /// Returns an [`Err`] if `src_idx` is out of the bounds of the array, or if the size
    /// of the array + `n_shifts` is greater than the maximum size of the array. Otherwise
    /// returns an empty [`Ok`].
    fn shift_right(&mut self, src_idx:usize, n_shifts:usize) -> Result<(),TracingPageError> {
        if (src_idx >= self.size) | (self.size+n_shifts > METADATA_ARRAY_SIZE) {
            return Err(TracingPageError::RightShiftPreconditions);
        }
        let dst_idx = src_idx + n_shifts;
        let end_src_idx = src_idx + (self.size - src_idx);
        self.metadata.copy_within(src_idx..end_src_idx, dst_idx);
        self.size += n_shifts;
        Ok(())
    }

    /// Shrinks the number of valid entries in the array to the given number
    /// and resets all the exceeding entries to the default
    ///
    /// ## Panics
    ///
    /// Panics if `to_size` is out-of-bounds.
    fn shrink(&mut self, to_size:usize) {
        let count = self.size - to_size;
        unsafe {
            let dst = self.metadata.as_mut_ptr().add(to_size);
            memset_defaulted(dst, count);
        }
        self.size = to_size;
    }

    /// Checks whether the given [`Metadata`] can be pushed to the array
    ///
    /// ## Returns
    ///
    /// Returns `false` if pushing would not respect the order of the entries,
    /// `true` otherwise.
    fn can_push(&self, md:&Metadata) -> bool {
        self.is_empty() | (md.higher_paddr() <= self.lower_paddr())
    }

    /// Inserts the given [`Metadata`] entry at the first position of the array
    ///
    /// This function tries to merge the new entry with the entry that is currently
    /// in the first position, if they're both free and are strictly contiguous to each
    /// other, otherwise pushes it and moves all the following entries one position
    /// toward the right, returning the last entry if they exceeds the maximum length
    /// of the array.
    ///
    /// ## Returns
    ///
    /// Returns an [`Err`] if pushing does not respect the entries order. Otherwise returns
    /// an [`Ok`] containing a [`Some`] with a [`Metadata`] entry if the array is full and
    /// the last entry needs to be moved to the next [`TracingPage`], or [`None`] otherwise.
    pub(in crate::memory::paging::tracing)
    fn try_push(&mut self, md:Metadata) -> Result<Option<Metadata>,TracingPageError> {
        if !self.can_push(&md) {
            return Err(TracingPageError::PushPreconditions);
        }
        Ok(self.push(md))
    }

    /// Inserts the given [`Metadata`] entry at the first position of the array
    ///
    /// This function tries to merge the new entry with the entry that is currently
    /// in the first position, if they're both free and are strictly contiguous to each
    /// other, otherwise pushes it and moves all the following entries one position
    /// toward the right, returning the last entry if they exceeds the maximum length
    /// of the array.
    ///
    /// ## Warning
    ///
    /// This function doesn't check whether the order of the entries is respected before
    /// pushing, use with great care.
    ///
    /// ## Returns
    ///
    /// Returns [`Some`] containing a [`Metadata`] entry if the array is full and the last
    /// entry needs to be moved to the next [`TracingPage`], or [`None`] otherwise.
    fn push(&mut self, md:Metadata) -> Option<Metadata> {
        if self.is_empty() {
            self.append_unchecked(md);
            return None;
        }
        let both_free = md.is_free() & self.metadata[0].is_free();
        if both_free && self.metadata[0].is_contiguous_to(&md) {
            self.metadata[0] += (md.size(), Side::Left);
            return None;
        }
        let exceeding_md = match self.is_full() {
            true  => Some(self.pop_unchecked()),
            false => None,
        };
        self.insert_unchecked(0, md);
        exceeding_md
    }

    /// Checks whether the given [`Excess`] can be pushed to the array
    ///
    /// ## Returns
    ///
    /// Returns `false` if pushing would not respect the order of the entries,
    /// `true` otherwise.
    fn can_push_excess(&self, exc:&Excess) -> bool {
        match exc.size {
            1 => self.can_push(&exc.data[0]),
            2 => self.can_push(&exc.data[0]) & self.can_push(&exc.data[1]),
            _ => true,
        }
    }

    /// Pushes the [`Metadata`] entries contained in the given [`Excess`] to the array
    ///
    /// ## Returns
    ///
    /// Returns an [`Err`] if pushing fails. Otherwise returns an [`Ok`] containing an
    /// [`Excess`] if the array is full and some entries need to be moved to the next
    /// [`TracingPage`].
    pub(in crate::memory::paging::tracing)
    fn try_push_excess(&mut self, exc:Excess) -> Result<Excess,TracingPageError> {
        if !self.can_push_excess(&exc) {
            return Err(TracingPageError::PushPreconditions);
        }
        match exc.size {
            0 => Ok(Excess::none()),
            1 => Ok(self.push(exc.data[0]).into()),
            2 => {
                if self.is_empty() {
                    self.metadata[0] = exc.data[0];
                    self.metadata[1] = exc.data[1];
                    self.size += 2;
                    return Ok(Excess::none());
                }
                let both_free = exc.data[1].is_free() & self.metadata[0].is_free();
                if both_free && self.metadata[0].is_contiguous_to(&exc.data[1]) {
                    self.metadata[0] += (exc.data[1].size(), Side::Left);
                    let excess = match self.is_full() {
                        true  => Excess::single(self.pop_unchecked()),
                        false => Excess::none(),
                    };
                    self.insert_unchecked(0, exc.data[0]);
                    return Ok(excess);
                } else {
                    let excess = match self.is_full() {
                        false => Excess::none(),
                        true  => {
                            let aux = Excess::double(self.metadata[self.size-2], self.metadata[self.size-1]);
                            self.size -= 2; // need to do, or shift_right() will fail
                            aux
                        },
                    };
                    self.shift_right(0, 2)?;
                    self.metadata[0] = exc.data[0];
                    self.metadata[1] = exc.data[1];
                    return Ok(excess);
                }
            },
            _ => Err(TracingPageError::InternalFailure),
        }
    }

    /// Checks whether the given [`Metadata`] can be appended to the array
    ///
    /// ## Returns
    ///
    /// Returns `false` if appending would not respect the order of the entries,
    /// `true` otherwise.
    fn can_append(&self, md:&Metadata) -> bool {
        self.is_empty() || self.higher_paddr() <= md.lower_paddr()
    }

    /// Inserts the given [`Metadata`] entry at the last position of the array
    ///
    /// This function tries to merge the new entry with the entry that is currently
    /// in the last position, if they're both free and strictly contiguous to each
    /// other, otherwise tries to append it, returning it if the array exceeds its
    /// maximum length.
    ///
    /// ## Returns
    ///
    /// Returns an [`Err`] if appending does not respect the entries order. Otherwise
    /// returns an [`Ok`] containing a [`Some`] with a [`Metadata`] entry if the array
    /// is full and the entry itself needs to be moved to the next [`TracingPage`], or
    /// [`None`] otherwise.
    pub(in crate::memory::paging::tracing)
    fn try_append(&mut self, md:Metadata) -> Result<Option<Metadata>,TracingPageError> {
        if !self.can_append(&md) {
            return Err(TracingPageError::AppendPreconditions);
        }
        Ok(self.append(md))
    }

    /// Inserts the given [`Metadata`] entry at the last position of the array
    ///
    /// ## Warning
    ///
    /// This function desn't check whether the order of the entries is respected upon
    /// appending, use with great caution.
    ///
    /// ## Panics
    ///
    /// Panics if the array is full
    ///
    /// ## Returns
    ///
    /// Returns [`Some`] with a [`Metadata`] entry if the array is full and the entry
    /// itself needs to be moved to the next [`TracingPage`], or [`None`] otherwise.
    pub(in crate::memory::paging::tracing)
    fn append(&mut self, md:Metadata) -> Option<Metadata> {
        if !self.is_empty() {
            let last = self.last_index();
            let both_free = md.is_free() & self.metadata[last].is_free();
            if both_free && md.is_contiguous_to(&self.metadata[last]) {
                self.metadata[last] += (md.size(), Side::Right);
                return None;
            } else if self.is_full() {
                return Some(md);
            }
        }
        self.append_unchecked(md);
        None
    }

    /// Inserts the given [`Metadata`] entry at the last position of the array
    ///
    /// ## Warning
    ///
    /// This function desn't check whether the order of the entries is respected upon
    /// appending, nor whether the [`Metadata`] array is already full or whether the
    /// given element can be merged to the last entry, use with extreme caution.
    ///
    /// ## Panics
    ///
    /// Panics if the array is full.
    pub(in crate::memory::paging::tracing)
    fn append_unchecked(&mut self, md:Metadata) {
        self.metadata[self.size] = md;
        self.size += 1;
    }

    /// Inserts the given [`Metadata`] entry in the array
    ///
    /// The entry will be inserted in such a way that the ordering is respected.
    ///
    /// ## Returns
    ///
    /// If no error occurs, returns an [`Ok`] with an [`Excess`] of type none if
    /// the array wasn't full, or an [`Excess`] containing an entry that cannot be
    /// stored in this page anymore and needs to be moved to the next [`TracingPage`].
    pub(in crate::memory::paging::tracing)
    fn insert(&mut self, md:Metadata) -> Result<Excess,TracingPageError> {
        if self.can_append(&md) {
            return Ok(self.append(md).into());
        } else if self.can_push(&md) {
            return Ok(self.push(md).into());
        }
        let md_lower_paddr = md.lower_paddr();
        for i in 0..self.size {
            let curr_md_higher_paddr = self.metadata[i].higher_paddr();
            if curr_md_higher_paddr < md_lower_paddr {
                continue;
            }
            if md.is_free() & self.metadata[i].is_free() {
                // try to merge with the current entry
                if md.is_contiguous_to(&self.metadata[i]) {
                    self.metadata[i] += (md.size(), Side::Right);
                    return Ok(Excess::none());
                } else if self.metadata[i].is_contiguous_to(&md) {
                    self.metadata[i] += (md.size(), Side::Left);
                    return Ok(Excess::none());
                }
            }
            let i = match curr_md_higher_paddr == md_lower_paddr {
                true  => i + 1, // insert afer the current
                false => i,
            };
            // insert at the current position
            let excess = match self.is_full() {
                true  => Excess::single(self.pop_unchecked()),
                false => Excess::none(),
            };
            self.insert_unchecked(i, md);
            return Ok(excess);
        }
        Err(TracingPageError::NotFound)
    }

    /// Inserts the given [`Metadata`] entry at the given index in the array
    ///
    /// ## Warning
    ///
    /// This function desn't check whether inserting the given element at the
    /// given index will actually preserve the order of the array, nor whether
    /// the element can be merged to the neighbour entries, nor whether the
    /// [`Metadata`] array is already full. Use with extreme caution.
    ///
    /// ## Panics
    ///
    /// Panics if the array is full.
    pub(in crate::memory::paging::tracing)
    fn insert_unchecked(&mut self, idx:usize, md:Metadata) {
        self.shift_right(idx, 1).ok_or_panic();
        self.metadata[idx] = md;
    }

    /// Removes the last [`Metadata`] entry from the array and returns it
    ///
    /// ## Returns
    ///
    /// Returns an [`Err`] if the array is empty, otherwise returns an [`Ok`]
    /// containing the last entry.
    pub(in crate::memory::paging::tracing)
    fn try_pop(&mut self) -> Result<Metadata,TracingPageError> {
        if self.is_empty() {
            return Err(TracingPageError::PopPreconditions);
        }
        Ok(self.pop_unchecked())
    }

    /// Removes the last [`Metadata`] entry from the array and returns it
    ///
    /// ## Warning
    ///
    /// This function doesn't check whether the array is empty or not, in which case
    /// an entry of type none is returned. Use with great care.
    ///
    /// ## Returns
    ///
    /// Returns the last entry.
    fn pop_unchecked(&mut self) -> Metadata {
        self.size -= 1;
        let md = self.metadata[self.size];
        self.metadata[self.size] = Metadata::default();
        md
    }

    /// Checks whether the [`Metadata`] at the given index exists in the array
    /// and can therefore be extracted from it
    fn can_extract(&self, idx:usize) -> bool {
        idx < self.size
    }

    /// Removes the [`Metadata`] entry at the given index from the array and
    /// returns it
    ///
    /// ## Returns
    ///
    /// Returns an [`Err`] if `idx` is out-of-bounds, otherwise returns an [`Ok`]
    /// holding the corresponding entry.
    pub(in crate::memory::paging::tracing)
    fn try_extract(&mut self, idx:usize) -> Result<Metadata, TracingPageError> {
        if !self.can_extract(idx) {
            return Err(TracingPageError::ExtractPreconditions);
        }
        Ok(self.extract(idx))
    }

    /// Removes the entry at the given index from the array and returns it
    ///
    /// ## Warning
    ///
    /// This function doesn't check whether the index is inside the bounds of
    /// the array, use with extreme caution.
    ///
    /// ## Panics
    ///
    /// Panics if the index is out-of-bounds.
    ///
    /// ## Returns
    ///
    /// Returns the [`Metadata`] entry at the given index
    pub(in crate::memory::paging::tracing)
    fn extract(&mut self, idx:usize) -> Metadata {
        if self.is_last_index(idx) {
            self.pop_unchecked()
        } else {
            self.extract_unchecked(idx)
        }
    }

    /// Removes the entry at the given index from the array and returns it
    ///
    /// ## Warning
    ///
    /// This function doesn't check whether the index is inside the bounds of
    /// the array, or if it is the last index. Use with extreme caution.
    ///
    /// ## Panics
    ///
    /// Panics if the index is out-of-bounds or if it is the last index.
    ///
    /// ## Returns
    ///
    /// Returns the [`Metadata`] entry at the given index.
    pub(in crate::memory::paging::tracing)
    fn extract_unchecked(&mut self, idx:usize) -> Metadata {
        let md = self.metadata[idx];
        self.shift_left(idx + 1, 1).ok_or_panic();
        self.size -= 1;
        self.metadata[self.size] = Metadata::default();
        md
    }

    /// Checks whether an undefined amount of space starting at the given
    /// [`PhysicalAddress`] is contained in the array and can thus theoretically
    /// be removed from it
    fn can_remove(&self, paddr:PhysicalAddress) -> bool {
        self.contains_paddr_strict(paddr)
    }

    /// Removes the given amount of space from the array
    ///
    /// The corresponding amount of entries is removed and/or updated.
    ///
    /// ## Returns
    ///
    /// Returns an [`Err`] if the requested space does not exists in the array,
    /// otherwise returns an [`Ok`] containing a [`Reminder`] with the left-over space.
    pub(in crate::memory::paging::tracing)
    fn try_remove(&mut self, paddr:PhysicalAddress, size:u64) -> Result<Reminder,TracingPageError> {
        if !self.can_remove(paddr) {
            return Err(TracingPageError::RemovePreconditions);
        }
        self.remove(paddr, size)
    }

    /// Removes the [`Metadata`] of one or more contiguous entries from the [`TracingPage`]
    ///
    /// The given [`PhysicalAddress`] represents the starting memory address (it may coincide
    /// with the beginning of an entry or be in the middle of it, and the same is for the end)
    /// and the given `size` represents the total size that need to be removed (it may
    /// correspond to multiple consecutive entries)
    ///
    /// ## Returns
    ///
    /// Returns an [`Err`] if the size to be removed overlap an entry marked as taken
    /// without entirely removing it (which means the request is to partially remove an
    /// allocated object). Otherwise returns an [`Ok`] with the size still left to remove.
    /// If the size left is greater than `0`, it means it must be removed from the next
    /// [`TracingPage`].
    fn remove(&mut self, mut paddr:PhysicalAddress, mut size:u64) -> Result<Reminder,TracingPageError> {
        use core::cmp::Ordering::*;
        let mut n_shifts = 0;
        for i in 0..self.size {
            if !self.metadata[i].contains(paddr) {
                if n_shifts > 0 {
                    // some entries have already been iterated, so if this entry does not
                    // contain the address it means something really wrong happened while
                    // managing this tracing page
                    return Err(TracingPageError::InternalFailure);
                }
                continue;
            }
            match self.metadata[i].info.partial_cmp(&size) {
                None => return Err(TracingPageError::EntryIsNone),
                Some(Equal) => {
                    // the entry will be complitely removed, regardless of whether it's marked
                    // as taken or as free
                    n_shifts += 1;
                    let next_i = i + 1;
                    if next_i < self.size {
                        self.shift_left(next_i, n_shifts)?;
                    }
                    self.shrink(self.size - n_shifts);
                    return Ok(Reminder::zero());
                },
                Some(Greater) => {
                    // the entry will be splitted and only the required size will be removed
                    if self.metadata[i].is_taken() {
                        // attempting to partially remove an allocation
                        return Err(TracingPageError::InvalidRequest);
                    }
                    let mut i = i;
                    if n_shifts > 0 {
                        self.shift_left(i, n_shifts)?; // the current entry must be shifted too
                        self.shrink(self.size - n_shifts);
                        i -= n_shifts;
                    }
                    let mut rem = Reminder::zero();
                    if self.metadata[i].lower_paddr() == paddr {
                        // removing the lower part
                        self.metadata[i].sub_assign((size, Side::Left));
                    } else if self.metadata[i].higher_paddr() == paddr + size {
                        // removing the higher part
                        self.metadata[i].sub_assign((size, Side::Right));
                    } else {
                        // removing the middle part
                        if n_shifts > 0 {
                            // some entries have already been iterated, so having to remove the middle
                            // part on an entry means something really wrong happened while managing
                            // this tracing page
                            return Err(TracingPageError::InternalFailure);
                        }
                        let lower_diff = paddr - self.metadata[i].lower_paddr();
                        let upper_md = self.metadata[i].sub((lower_diff + size, Side::Left));
                        self.metadata[i].set_size(lower_diff)?;
                        let next_i = i + 1;
                        if next_i == self.size {
                            if self.is_full() {
                                // the exceeding part of the current entry must be moved to the next page
                                return Ok(Reminder::positive(upper_md));
                            }
                            self.append_unchecked(upper_md);
                            return Ok(Reminder::zero());
                        } else if self.is_full() {
                            // the last entry must be moved to the next page
                            rem = Reminder::positive(self.pop_unchecked());
                        }
                        self.insert_unchecked(next_i, upper_md);
                    }
                    return Ok(rem);
                },
                Some(Less) => {
                    let next_i = i + 1;
                    if next_i < self.size && !self.metadata[next_i].is_contiguous_to(&self.metadata[i]) {
                        // cannot release multiple non-contiguous entries a part of the same allocation
                        return Err(TracingPageError::InvalidRequest);
                    }
                    if self.metadata[i].lower_paddr() == paddr {
                        // this entry will be completely wiped out
                        let current_size = self.metadata[i].size();
                        paddr += current_size;
                        size -= current_size;
                        if next_i == self.size {
                            let mut md = self.pop_unchecked();
                            md -= (current_size, Side::Left);
                            md.set_size(size)?;
                            if n_shifts > 0 {
                                self.shrink(self.size - n_shifts);
                            }
                            return Ok(Reminder::negative(md));
                        }
                        n_shifts += 1;
                    } else if n_shifts > 0 {
                        // some entries have already been iterated, so if the beginning of this entry
                        // does not match the address it means something really wrong happened while
                        // managing this tracing page
                        return Err(TracingPageError::InternalFailure);
                    } else {
                        // only the ending part of this entry need to be wiped out
                        if self.metadata[i].is_taken() {
                            // partially releasing an allocation is not supposed to happen
                            return Err(TracingPageError::InvalidRequest);
                        }
                        let size_diff = paddr - self.metadata[i].lower_paddr();
                        self.metadata[i].sub_assign((size_diff, Side::Right));
                        size -= size_diff;
                    }
                },
            }
        }
        Err(TracingPageError::NotFound)
    }

    /// Removes the [`Metadata`] entry at the given index from the array
    ///
    /// ## Warning
    ///
    /// This function doesn't check whether the array is empty or not, nor whether
    /// the given index is inside the bounds of the array. Use with extreme caution.
    ///
    /// ## Panics
    ///
    /// Panics if the array is empty or if the index is greater than the size of the
    /// array itself.
    fn remove_entry_unchecked(&mut self, idx:usize) {
        let next_idx = idx + 1;
        if next_idx < self.size {
            self.shift_left(next_idx, 1).ok_or_panic();
        }
        self.size -= 1;
        self.metadata[self.size] = Metadata::default();
    }

    /// Marks the [`Metadata`] entry corresponding to the given physical address as taken
    ///
    /// The given [`PhysicalAddress`] should most likely match the beginning of an entry,
    /// but its size is not supposed to match its entire size (and most likely won't).
    ///
    /// ## Returns
    ///
    /// Returns an [`Err`] if the given [`PhysicalAddress`] is not contained in the array,
    /// if the [`Metadata`] is already marked as taken or if its free size is less than `size`.
    /// Otherwise returns [`Ok`] containing an [`Excess`] with the exceeding entries which
    /// must be moved to the next [`TracingPage`].
    pub(in crate::memory::paging::tracing)
    fn take(&mut self, paddr:PhysicalAddress, size:u64) -> Result<Excess,TracingPageError> {
        use core::cmp::Ordering::*;
        for i in 0..self.size {
            if !self.metadata[i].contains(paddr) {
                continue;
            }
            if self.metadata[i].is_taken() {
                return Err(TracingPageError::EntryIsTaken);
            }
            match self.metadata[i].info.partial_cmp(&size) {
                None => {
                    return Err(TracingPageError::EntryIsNone)
                },
                Some(Less) => {
                    return Err(TracingPageError::EntrySizeMismatch)
                },
                Some(Equal) => {
                    self.metadata[i].set_taken();
                    return Ok(Excess::none());
                },
                Some(Greater) => {
                    // the entry will be splitted and only the needed part will be taken
                    if self.metadata[i].lower_paddr() == paddr {
                        // only taking the lower part of the entry
                        return self.take_entry_lower_higher(i, paddr, size, Side::Left);
                    } else if self.metadata[i].higher_paddr() == paddr + size {
                        // only taking the higher part of the entry
                        return self.take_entry_lower_higher(i, paddr, size, Side::Right);
                    } else {
                        // only taking the middle part of the entry
                        return self.take_entry_middle(i, paddr, size);
                    }
                },
            }
        }
        Err(TracingPageError::NotFound)
    }

    fn take_entry_lower_higher(&mut self, i:usize, paddr:PhysicalAddress, size:u64, side:Side) -> Result<Excess,TracingPageError> {
        let excess_md = match side {
            Side::Left => {
                // taking the lower part of the entry
                let mut higher_md = self.metadata[i];
                self.metadata[i].set_size(size)?;
                self.metadata[i].set_taken();
                higher_md -= (size, Side::Left);
                higher_md
            },
            Side::Right => {
                // taking the higher part of the entry
                let mut higher_md = self.metadata[i];
                self.metadata[i] -= (size, Side::Right);
                higher_md -= (self.metadata[i].size(), Side::Left);
                higher_md.set_taken();
                higher_md
            },
        };
        let next_i = i + 1;
        if self.is_full() {
            if next_i == self.size {
                return Ok(Excess::single(excess_md));
            } else {
                let excess = Excess::single(self.pop_unchecked());
                self.insert_unchecked(next_i, excess_md);
                return Ok(excess);
            }
        } else {
            if next_i == self.size {
                self.append_unchecked(excess_md);
            } else {
                self.insert_unchecked(next_i, excess_md);
            }
            return Ok(Excess::none());
        }
    }

    fn take_entry_middle(&mut self, i:usize, paddr:PhysicalAddress, size:u64) -> Result<Excess,TracingPageError> {
        // taking the middle part of the entry
        let lower_diff = paddr - self.metadata[i].lower_paddr();
        let mut middle_md = self.metadata[i];
        middle_md -= (lower_diff, Side::Left);
        middle_md.set_size(size)?;
        middle_md.set_taken();
        let mut higher_md = self.metadata[i];
        higher_md -= (lower_diff+size, Side::Left);
        self.metadata[i].set_size(lower_diff)?;

        let next_i = i + 1;
        let unused_size = self.unused_size();
        if unused_size == 0 {
            if next_i == self.size {
                return Ok(Excess::double(middle_md, higher_md));
            } else if self.is_last_index(next_i) {
                let excess_md = self.metadata[next_i];
                self.metadata[next_i] = middle_md;
                return Ok(Excess::double(higher_md, excess_md));
            } else {
                let excess = {
                    let md = self.pop_unchecked();
                    Excess::double(self.pop_unchecked(), md)
                };
                self.shift_right(next_i, 2)?;
                self.metadata[next_i] = middle_md;
                self.metadata[next_i+1] = higher_md;
                return Ok(excess);
            }
        } else if unused_size == 1 {
            if next_i == self.size {
                self.append_unchecked(middle_md);
                return Ok(Excess::single(higher_md));
            } else if self.is_last_index(next_i) {
                let excess_md = self.metadata[next_i];
                self.metadata[next_i] = middle_md;
                self.append_unchecked(higher_md);
                return Ok(Excess::single(excess_md));
            } else {
                let excess = {
                    let md = self.pop_unchecked();
                    Excess::double(self.pop_unchecked(), md)
                };
                self.shift_right(next_i, 2)?;
                self.metadata[next_i] = middle_md;
                self.metadata[next_i+1] = higher_md;
                return Ok(excess);
            }
        } else {
            if next_i == self.size {
                self.append_unchecked(middle_md);
                self.append_unchecked(higher_md);
            } else {
                self.shift_right(next_i, 2)?;
                self.metadata[next_i] = middle_md;
                self.metadata[next_i+1] = higher_md;
            }
            return Ok(Excess::none());
        }
    }

    /// Marks the [`Metadata`] entry corresponding to the given [`PhysicalAddress`] as free
    ///
    /// ## Returns
    ///
    /// Returns an [`Err`] if the given [`PhysicalAddress`] is not contained in the array,
    /// if the [`Metadata`] is already marked as free, if its size is less than `size`.
    /// Otherwise returns an [`Ok`] containing the size of the entry.
    pub(in crate::memory::paging::tracing)
    fn drop(&mut self, paddr:PhysicalAddress) -> Result<u64,TracingPageError> {
        for i in 0..self.size {
            if self.metadata[i].contains(paddr) {
                if self.metadata[i].lower_paddr() != paddr {
                    return Err(TracingPageError::DropPreconditions);
                } else if self.metadata[i].is_free() { // leave last
                    return Err(TracingPageError::EntryIsFree);
                }
                let size = self.metadata[i].size();
                self.free_and_try_merge(i)?;
                return Ok(size);
            }
        }
        Err(TracingPageError::NotFound)
    }

    /// Resizes the [`Metadata`] entry corresponding to the given [`PhysicalAddress`] by
    /// increasing or reducing its size
    ///
    /// If the new size equals the old size, the operation is a no-op.
    ///
    /// ## Returns
    ///
    /// Returns an [`Err`] if the entry to be resized is not marked as taken, if `new_size`
    /// is bigger than the size of the entry or if the next entry (if any) is not free.
    /// If no error occurs, returns an [`Ok`] containing a [`Reminder`] with the left-over
    /// size that need to be added to or removed from the next [`TracingPage`].
    pub(in crate::memory::paging::tracing)
    fn resize(&mut self, paddr:PhysicalAddress, new_size:u64) -> Result<Reminder,TracingPageError> {
        for i in 0..self.size {
            if self.metadata[i].contains(paddr) {
                if self.metadata[i].lower_paddr() != paddr {
                    return Err(TracingPageError::ResizePreconditions);
                } else if self.metadata[i].is_free() { // leave last
                    return Err(TracingPageError::EntryIsFree);
                }
                return self.resize_entry(i, new_size);
            }
        }
        Err(TracingPageError::NotFound)
    }

    /// Resize the entry at the given index
    fn resize_entry(&mut self, i:usize, new_size:u64) -> Result<Reminder,TracingPageError> {
        let old_size = self.metadata[i].size();
        if old_size > new_size {
            return self.resize_entry_shrink(i, new_size);
        } else if old_size < new_size {
            return self.resize_entry_expand(i, old_size, new_size);
        } else {
            return Ok(Reminder::zero());
        }
    }

    /// Resizes the entry at the given index by shrinking it
    fn resize_entry_shrink(&mut self, i:usize, new_size:u64) -> Result<Reminder, TracingPageError> {
        let next_i = i + 1;
        let mut higher_md = self.metadata[i];
        higher_md -= (new_size, Side::Left);
        higher_md.set_free();
        self.metadata[i] -= (higher_md.size(), Side::Right);
        if next_i == self.size {
            if self.is_full() {
                return Ok(Reminder::positive(higher_md));
            } else {
                self.append_unchecked(higher_md);
                return Ok(Reminder::zero());
            }
        } else if self.metadata[next_i].is_free() && self.metadata[next_i].is_contiguous_to(&higher_md) {
            self.metadata[next_i] += (higher_md.size(), Side::Left);
            return Ok(Reminder::zero());
        } else {
            let excess_md = match self.is_full() {
                true  => Some(self.pop_unchecked()),
                false => None,
            };
            self.insert_unchecked(next_i, higher_md);
            return match excess_md {
                Some(md) => Ok(Reminder::positive(md)),
                None => Ok(Reminder::zero()),
            };
        }
    }

    /// Resizes the entry at the given index by expanding it
    fn resize_entry_expand(&mut self, i:usize, old_size:u64, new_size:u64) -> Result<Reminder, TracingPageError> {
        let next_i = i + 1;
        if next_i == self.size {
            if !self.is_full() {
                return Err(TracingPageError::ResizePreconditions);
            }
        } else if self.metadata[next_i].is_taken() || !self.metadata[next_i].is_contiguous_to(&self.metadata[i]) {
            return Err(TracingPageError::ResizePreconditions);
        }
        let diff_size = new_size - old_size;
        if next_i == self.size {
            // the pre-conditions ensure the metadata array is full in this case
            self.metadata[i] += (diff_size, Side::Right);
            let mut missing_md = self.metadata[i];
            missing_md >>= diff_size;
            missing_md.set_size(diff_size)?;
            missing_md.set_free();
            return Ok(Reminder::negative(missing_md));
        } else {
            // the pre-conditions ensure the next entry is free and contiguous in this case
            let next_md_size = self.metadata[next_i].size();
            if next_md_size < diff_size {
                return Err(TracingPageError::EntrySizeMismatch);
            }
            self.metadata[i] += (diff_size, Side::Right);
            if next_md_size > diff_size {
                self.metadata[next_i] -= (diff_size, Side::Left);
                return Ok(Reminder::zero());
            } else {
                self.remove_entry_unchecked(next_i);
                return Ok(Reminder::zero());
            }
        }
    }

    /// Splits the [`Metadata`] entry at the given index into two entries
    ///
    /// The first entry will have the size defined by `split_size` and the second
    /// will have the size that remains. Both entries will mantain their type.
    ///
    /// ## Returns
    ///
    /// Returns an [`Err`] if the [`Metadata`] is not initialized, if `split_idx`
    /// is out-of-bounds or if `split_size` is bigger than the size of the entry.
    /// Otherwise returns an [`Ok`] containing a [`Some`] if the array is full and
    /// the returned entry must be moved to the next page, or a [`None`] otherwise.
    fn split(&mut self, split_idx:usize, split_size:u64) -> Result<Option<Metadata>,TracingPageError> {
        if (split_idx >= self.size) | self.metadata[split_idx].is_none() {
            return Err(TracingPageError::SplitPreconditions);
        }
        let entry_size = self.metadata[split_idx].size();
        if entry_size < split_size {
            return Err(TracingPageError::EntrySizeMismatch);
        } else if entry_size == split_size {
            return Ok(None);
        }
        let mut md = self.metadata[split_idx];
        self.metadata[split_idx].set_size(split_size)?;
        md -= (split_size, Side::Left);
        let next_idx = split_idx + 1;
        if next_idx == self.size {
            if self.is_full() {
                return Ok(Some(md));
            } else {
                self.append_unchecked(md);
                return Ok(None);
            }
        } else {
            let excess_md = match self.is_full() {
                true  => Some(self.pop_unchecked()),
                false => None,
            };
            self.shift_right(next_idx, 1)?;
            self.metadata[next_idx] = md;
            return Ok(excess_md);
        }
    }

    /// Splits the entry at the given index into two entries
    ///
    /// The first entry will have the size defined by `split_size` and the second
    /// will have the size that remains. Both entries will mantain their type.
    ///
    /// ## Warning
    ///
    /// This function does not perform any kind of check before splitting. Use with
    /// great care.
    ///
    /// ## Panics
    ///
    /// Panics if `split_idx` is out-of-bounds, if the array is already full or if
    /// the entry at `split_idx` is not initialized.
    fn split_unchecked(&mut self, split_idx:usize, split_size:u64) {
        let mut lower_md = self.metadata[split_idx];
        lower_md.set_size(split_size).ok_or_panic();
        self.metadata[split_idx] -= (split_size, Side::Left);
        self.shift_right(split_idx, 1).ok_or_panic();
        self.metadata[split_idx] = lower_md;
    }

    /// Marks the [`Metadata`] entry at the given index as free and tries to merge it
    /// with the next and previous entries, if they're of free type and strictly
    /// contiguous to it
    ///
    /// ## Returns
    ///
    /// Returns an [`Err`] if `merge_idx` is out-of-bounds or the corresponding
    /// entry is not initialized, or if another error occurs in a sub-call.
    /// Otherwise returns an empty [`Ok`].
    fn free_and_try_merge(&mut self, merge_idx:usize) -> Result<(),TracingPageError> {
        if (merge_idx >= self.size) | self.metadata[merge_idx].is_none() {
            return Err(TracingPageError::MergePreconditions);
        }
        let mut md = self.metadata[merge_idx];
        md.set_free();
        let mut src_idx = merge_idx;
        let mut dst_idx = merge_idx;
        if merge_idx > 0 {
            let prev_idx = merge_idx-1;
            let pmd = &self.metadata[prev_idx];
            if pmd.is_free() && md.is_contiguous_to(pmd) {
                md.paddr = pmd.paddr;
                md.laddr = pmd.laddr;
                md.info += pmd.size();
                dst_idx -= 1;
            }
        }
        let next_idx = merge_idx+1;
        if next_idx < self.size {
            let nmd = &self.metadata[next_idx];
            if nmd.is_free() && nmd.is_contiguous_to(&md) {
                md.info += nmd.size();
                src_idx += 1;
            }
        }
        if src_idx == dst_idx {
            self.metadata[merge_idx].set_free();
            return Ok(());
        }
        let n_shifts = src_idx - dst_idx;
        self.shift_left(src_idx, n_shifts)?;
        self.shrink(self.size - n_shifts);
        self.metadata[dst_idx] = md;
        Ok(())
    }
}

#[cfg(feature="unit_tests")]
impl TracingPage {
    pub (in crate::memory::paging::tracing)
    fn fill_alternate(&mut self, mut paddr:PhysicalAddress, mut laddr:LogicalAddress, md_size:u64) {
        self.size = METADATA_ARRAY_SIZE;
        for i in 0..METADATA_ARRAY_SIZE {
            if i%2 == 0 {
                self.metadata[i] = Metadata::new_taken(paddr, laddr, md_size);
            } else {
                self.metadata[i] = Metadata::new_free(paddr, laddr, md_size);
            }
            paddr += md_size;
            laddr += md_size;
        }
    }

    pub (in crate::memory::paging::tracing)
    fn fill_taken(&mut self, mut paddr:PhysicalAddress, mut laddr:LogicalAddress, md_size:u64) {
        self.size = METADATA_ARRAY_SIZE;
        for i in 0..METADATA_ARRAY_SIZE {
            self.metadata[i] = Metadata::new_taken(paddr, laddr, md_size);
            paddr += md_size;
            laddr += md_size;
        }
    }

    pub (in crate::memory::paging::tracing)
    fn clear(&mut self) {
        self.size = 0;
        unsafe {
            memset_defaulted(self.metadata.as_mut_ptr(), METADATA_ARRAY_SIZE);
        }
    }

    pub (in crate::memory::paging::tracing)
    fn entry_at(&self, idx:usize) -> &Metadata {
        &self.metadata[idx]
    }

    pub (in crate::memory::paging::tracing)
    fn entry_at_mut(&mut self, idx:usize) -> &mut Metadata {
        &mut self.metadata[idx]
    }
}

impl Init<PhysicalAddress> for TracingPage {
    fn init(ptr:PhysicalAddress) {
        let page = Self::cast_mut(ptr);
        unsafe {
            (*page).size = 0;
            memset_defaulted((*page).metadata.as_mut_ptr(), METADATA_ARRAY_SIZE);
        }
    }
}

impl Init<LogicalAddress> for TracingPage {
    fn init(ptr:LogicalAddress) {
        let page = Self::cast_mut(ptr);
        unsafe {
            (*page).size = 0;
            memset_defaulted((*page).metadata.as_mut_ptr(), METADATA_ARRAY_SIZE);
        }
    }
}

impl Cast<PhysicalAddress> for TracingPage {
    fn cast(ptr:PhysicalAddress) -> *const Self {
        ptr.as_ptr()
    }

    fn cast_mut(ptr:PhysicalAddress) -> *mut Self {
        ptr.as_ptr_mut()
    }
}

impl Cast<LogicalAddress> for TracingPage {
    fn cast(ptr:LogicalAddress) -> *const Self {
        ptr.as_ptr()
    }

    fn cast_mut(ptr:LogicalAddress) -> *mut Self {
        ptr.as_ptr_mut()
    }
}
