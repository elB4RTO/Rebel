use crate::memory::MemoryOwner;
use crate::memory::paging::*;
use crate::memory::paging::iterators::{
    KERNEL_ALLOCATIONS_PML4T_INDEX_RANGE,
    USER_ALLOCATIONS_PML4T_INDEX_RANGE,
    KERNEL_ALLOCATIONS_PML4T_OFFSET_RANGE,
    USER_ALLOCATIONS_PML4T_OFFSET_RANGE,
    TRACING_TABLES_PML4T_OFFSET_RANGE,
    KERNEL_TRACING_TABLES_PDPT_OFFSET_RANGE,
    USER_TRACING_TABLES_PDPT_OFFSET_RANGE,
    Duplicate,
};

use core::ops::Range;

/// The size of an index increment needed to reach the next entry in a page table
const
TABLE_INDEX_INCREMENT_SIZE  : u64 = 1;

/// The size of an offset increment needed to reach the next entry in a page table
const
TABLE_OFFSET_INCREMENT_SIZE : u64 = 0x008;


pub(in crate::memory)
trait Indexer {
    /// Sets the iteration to proceed in reverse order
    fn reverse(&mut self);

    /// Restarts the index, setting it to the start of the range
    fn restart(&mut self);

    /// Terminates the index, setting it to the end of the range
    fn terminate(&mut self);

    /// Increments the index by one unit
    fn increment(&mut self);

    /// Checks whether the current index is inside the range of valid indexes
    ///
    /// ## Returns
    ///
    /// Returns `true` if the index is inside the bounds defined by the range,
    /// `false` otherwise.
    fn is_inside_bounds(&self) -> bool;

    /// Checks whether the next index will be inside the range of valid indexes
    ///
    /// ## Returns
    ///
    /// Returns `true` if the index is inside the bounds defined by the range,
    /// `false` otherwise.
    fn has_next(&self) -> bool;

    /// Returns the current index
    ///
    /// ## Returns
    ///
    /// Returns [`Some`] containing the index if the index itself is inside the bounds
    /// defined by the range, otherwise returns [`None`].
    fn get(&self) -> Option<u64>;


    /// Returns the current index
    ///
    /// ## Warning
    ///
    /// This function does not check whether the index is inside the bounds
    /// defined by the range or not.
    ///
    /// ## Returns
    ///
    /// Returns the index
    fn get_unchecked(&self) -> u64;

    /// Returns the next index in the range
    ///
    /// ## Returns
    ///
    /// Returns [`Some`] containing the index if the index itself is inside the bounds
    /// defined by the range, otherwise returns [`None`]
    fn next(&mut self) -> Option<u64>;

    /// Returns the next index in the range
    ///
    /// ## Warning
    ///
    /// This function does not check whether the index is inside the bounds
    /// defined by the range or not.
    ///
    /// ## Returns
    ///
    /// Returns the index
    fn next_unchecked(&mut self) -> u64;
}


/// Used to iterate through the entries of a page table using indexes
pub(in crate::memory)
struct PageTableEntryIndex {
    /// The current index
    current_index : u64,
    /// The next index
    next_index : u64,
    /// The range of valid indexes
    range : Range<u64>, // [start, end)
    /// Whether the iteration is in reverse order
    reversed : bool,
}

impl PageTableEntryIndex {
    /// Creates a [`PageTableEntryIndex`] specific for the allocation tables only,
    /// limited by the bounds defined by the given [`PageTableType`] and [`MemoryOwner`]
    pub(in crate::memory) const
    fn allocations(table_type:PageTableType, owner:MemoryOwner) -> Self {
        use PageTableType::*;
        match table_type {
            PageMapLevel4Table => match owner {
                MemoryOwner::Kernel => Self::range(KERNEL_ALLOCATIONS_PML4T_INDEX_RANGE),
                MemoryOwner::User => Self::range(USER_ALLOCATIONS_PML4T_INDEX_RANGE),
            },
            PageDirectoryPointerTable => Self::unbounded(),
            PageDirectoryTable => Self::unbounded(),
            PageTable => Self::unbounded(),
        }
    }

    /// Creates a [`PageTableEntryIndex`] specific for the allocation tables only,
    /// limited by the bounds defined by the given [`PageTableType`] and [`MemoryOwner`],
    /// with the index set to `entry_index`
    pub(in crate::memory) const
    fn allocations_with_index(entry_index:u64, table_type:PageTableType, owner:MemoryOwner) -> Self {
        Self::allocations(table_type, owner).with_index(entry_index)
    }

    /// Creates a [`PageTableEntryIndex`] specific for the tracing tables only,
    /// limited by the bounds defined by the given [`PageTableType`] and [`MemoryOwner`]
    pub(in crate::memory) const
    fn tracing(table_type:PageTableType, owner:MemoryOwner) -> Result<Self,PagingError> {
        use PageTableType::*;
        match table_type {
            PageMapLevel4Table => Ok(Self::range(TRACING_TABLES_PML4T_OFFSET_RANGE)),
            PageDirectoryPointerTable => match owner {
                MemoryOwner::Kernel => Ok(Self::range(KERNEL_TRACING_TABLES_PDPT_OFFSET_RANGE)),
                MemoryOwner::User => Ok(Self::range(USER_TRACING_TABLES_PDPT_OFFSET_RANGE)),
            },
            PageDirectoryTable => Ok(Self::unbounded()),
            PageTable => Err(PagingError::InvalidRequest),
        }
    }

    /// Creates an un-bounded [`PageTableEntryIndex`]
    pub(in crate::memory) const
    fn unbounded() -> Self {
        Self::range(FIRST_ENTRY_INDEX..LIMIT_ENTRY_INDEX)
    }

    /// Creates an un-bounded [`PageTableEntryIndex`], with the index set to `entry_index`
    pub(in crate::memory) const
    fn unbounded_with_index(entry_index:u64) -> Self {
        Self::unbounded().with_index(entry_index)
    }

    /// Creates a [`PageTableEntryIndex`] limited by the given [`Range`]
    const
    fn range(start_end:Range<u64>) -> Self {
        Self {
            current_index : start_end.start,
            next_index    : start_end.start,
            range         : start_end,
            reversed      : false,
        }
    }

    /// Sets the index to `entry_index` and returns the item
    ///
    /// ## Warning
    ///
    /// After using this function, a call to [`Indexer::next()`] will return
    /// the just set index. Only use on a newly created item.
    const
    fn with_index(mut self, entry_index:u64) -> Self {
        self.current_index = entry_index;
        self.next_index = entry_index;
        self
    }

    /// Returns the _current index_
    ///
    /// ## Warning
    ///
    /// This function does not perform any bound checking on the current index.
    ///
    /// ## Note
    ///
    /// Consider using [`Indexer::get()`] for a checked operation.
    const
    fn index(&self) -> u64 {
        self.current_index
    }

    /// Returns the _current index_ converted to an offset
    ///
    /// ## Warning
    ///
    /// This function does not perform any bound checking on the current index.
    pub(in crate::memory::paging) const
    fn offset(&self) -> u64 {
        self.current_index * TABLE_OFFSET_INCREMENT_SIZE
    }

    /// Checks whether the given `index` is inside the bounds defined by the range
    const
    fn check_bounds_of(&self, index:u64) -> bool {
        index < self.range.end
    }
}

impl Indexer for PageTableEntryIndex {
    fn reverse(&mut self) {
        self.reversed ^= true;
    }

    fn restart(&mut self) {
        self.current_index = self.range.start;
        self.next_index = self.range.start;
    }

    fn terminate(&mut self) {
        self.current_index = self.range.end;
        self.next_index = self.range.end;
    }

    fn increment(&mut self) {
        self.current_index = self.next_index;
        self.next_index += TABLE_INDEX_INCREMENT_SIZE;
    }

    fn is_inside_bounds(&self) -> bool {
        self.check_bounds_of(self.current_index)
    }

    fn has_next(&self) -> bool {
        self.check_bounds_of(self.next_index)
    }

    fn get(&self) -> Option<u64> {
        match self.is_inside_bounds() {
            true  => Some(self.get_unchecked()),
            false => None,
        }
    }

    fn get_unchecked(&self) -> u64 {
        match self.reversed {
            false => self.current_index,
            true  => self.range.end - self.current_index,
        }
    }

    fn next(&mut self) -> Option<u64> {
        if self.has_next() {
            return Some(self.next_unchecked());
        }
        self.current_index = self.next_index;
        None
    }

    fn next_unchecked(&mut self) -> u64 {
        self.increment();
        match self.reversed {
            false => self.current_index,
            true  => self.range.end - self.current_index,
        }
    }
}

impl Duplicate for PageTableEntryIndex {
    fn duplicate(&self) -> Self {
        Self {
            current_index : self.current_index,
            next_index    : self.next_index,
            range         : self.range.start..self.range.end,
            reversed      : self.reversed,
        }
    }

    fn duplicate_and_repeat(&self) -> Self {
        Self {
            current_index : self.current_index,
            next_index    : self.current_index,
            range         : self.range.start..self.range.end,
            reversed      : self.reversed,
        }
    }

    fn duplicate_and_advance(&self) -> Self {
        Self {
            current_index : self.next_index,
            next_index    : self.next_index,
            range         : self.range.start..self.range.end,
            reversed      : self.reversed,
        }
    }
}

impl From<PageTableEntryOffset> for PageTableEntryIndex {
    fn from(offset:PageTableEntryOffset) -> Self {
        Self{
            current_index : offset.current_offset / TABLE_OFFSET_INCREMENT_SIZE,
            next_index    : offset.next_offset / TABLE_OFFSET_INCREMENT_SIZE,
            range         : (offset.range.start / TABLE_OFFSET_INCREMENT_SIZE)..(offset.range.end / TABLE_OFFSET_INCREMENT_SIZE),
            reversed      : offset.reversed,
        }
    }
}


/// Used to iterate through the entries of a page table using offsets
pub(in crate::memory)
struct PageTableEntryOffset {
    /// The current offset
    current_offset : u64,
    /// The next offset
    next_offset : u64,
    /// The range of valid offsets
    range : Range<u64>, // [start, end)
    /// Whether the iteration is in reverse order
    reversed : bool,
}

impl PageTableEntryOffset {
    /// Creates a [`PageTableEntryOffset`] specific for the allocation tables only,
    /// limited by the bounds defined by the given [`PageTableType`] and [`MemoryOwner`]
    pub(in crate::memory) const
    fn allocations(table_type:PageTableType, owner:MemoryOwner) -> Self {
        use PageTableType::*;
        match table_type {
            PageMapLevel4Table => match owner {
                MemoryOwner::Kernel => Self::range(KERNEL_ALLOCATIONS_PML4T_OFFSET_RANGE),
                MemoryOwner::User => Self::range(USER_ALLOCATIONS_PML4T_OFFSET_RANGE),
            },
            PageDirectoryPointerTable => Self::unbounded(),
            PageDirectoryTable => Self::unbounded(),
            PageTable => Self::unbounded(),
        }
    }

    /// Creates a [`PageTableEntryOffset`] specific for the allocation tables only,
    /// limited by the bounds defined by the given [`PageTableType`] and [`MemoryOwner`],
    /// with the offset set to `entry_offset`
    pub(in crate::memory) const
    fn allocations_with_offset(entry_offset:u64, table_type:PageTableType, owner:MemoryOwner) -> Self {
        Self::allocations(table_type, owner).with_offset(entry_offset)
    }

    /// Creates a [`PageTableEntryOffset`] specific for the tracing tables only,
    /// limited by the bounds defined by the given [`PageTableType`] and [`MemoryOwner`]
    pub(in crate::memory) const
    fn tracing(table_type:PageTableType, owner:MemoryOwner) -> Result<Self,PagingError> {
        use PageTableType::*;
        match table_type {
            PageMapLevel4Table => Ok(Self::range(TRACING_TABLES_PML4T_OFFSET_RANGE)),
            PageDirectoryPointerTable => match owner {
                MemoryOwner::Kernel => Ok(Self::range(KERNEL_TRACING_TABLES_PDPT_OFFSET_RANGE)),
                MemoryOwner::User => Ok(Self::range(USER_TRACING_TABLES_PDPT_OFFSET_RANGE)),
            },
            PageDirectoryTable => Ok(Self::unbounded()),
            PageTable => Err(PagingError::InvalidRequest),
        }
    }

    /// Creates an un-bounded [`PageTableEntryOffset`]
    pub(in crate::memory) const
    fn unbounded() -> Self {
        Self::range(FIRST_ENTRY_OFFSET..LIMIT_ENTRY_OFFSET)
    }

    /// Creates an un-bounded [`PageTableEntryOffset`], with the offset set to `entry_offset`
    pub(in crate::memory) const
    fn unbounded_with_offset(entry_offset:u64) -> Self {
        Self::unbounded().with_offset(entry_offset)
    }

    /// Creates a [`PageTableEntryOffset`] limited by the given [`Range`]
    const
    fn range(start_end:Range<u64>) -> Self {
        Self {
            current_offset : start_end.start,
            next_offset    : start_end.start,
            range          : start_end,
            reversed       : false,
        }
    }

    /// Sets the offset to `entry_offset` and returns the item
    ///
    /// ## Warning
    ///
    /// After using this function, a call to [`Indexer::next()`] will return
    /// the just set offset. Only use on a newly created item.
    const
    fn with_offset(mut self, entry_offset:u64) -> Self {
        self.current_offset = entry_offset;
        self.next_offset = entry_offset;
        self
    }

    /// Returns the _current offset_
    ///
    /// ## Warning
    ///
    /// This function does not perform any bound checking on the current offset.
    ///
    /// ## Note
    ///
    /// Consider using [`Indexer::get()`] for a checked operation.
    pub(in crate::memory::paging) const
    fn offset(&self) -> u64 {
        self.current_offset
    }

    /// Returns the _current offset_ converted to an index
    ///
    /// ## Warning
    ///
    /// This function does not perform any bound checking on the current offset.
    pub(in crate::memory::paging) const
    fn index(&self) -> u64 {
        self.current_offset / TABLE_OFFSET_INCREMENT_SIZE
    }

    /// Checks whether the given `offset` is inside the bounds defined by the range
    const
    fn check_bounds_of(&self, offset:u64) -> bool {
        offset < self.range.end
    }
}

impl Indexer for PageTableEntryOffset {
    fn reverse(&mut self) {
        self.reversed ^= true;
    }

    fn restart(&mut self) {
        self.current_offset = self.range.start;
        self.next_offset = self.range.start;
    }

    fn terminate(&mut self) {
        self.current_offset = self.range.end;
        self.next_offset = self.range.end;
    }

    fn increment(&mut self) {
        self.current_offset = self.next_offset;
        self.next_offset += TABLE_OFFSET_INCREMENT_SIZE;
    }

    fn is_inside_bounds(&self) -> bool {
        self.check_bounds_of(self.current_offset)
    }

    fn has_next(&self) -> bool {
        self.check_bounds_of(self.next_offset)
    }

    fn get(&self) -> Option<u64> {
        match self.is_inside_bounds() {
            true  => Some(self.get_unchecked()),
            false => None,
        }
    }

    fn get_unchecked(&self) -> u64 {
        match self.reversed {
            false => self.current_offset,
            true  => self.range.end - self.current_offset,
        }
    }

    fn next(&mut self) -> Option<u64> {
        if self.has_next() {
            return Some(self.next_unchecked());
        }
        self.current_offset = self.next_offset;
        None
    }

    fn next_unchecked(&mut self) -> u64 {
        self.increment();
        match self.reversed {
            false => self.current_offset,
            true  => self.range.end - self.current_offset,
        }
    }
}

impl Duplicate for PageTableEntryOffset {
    fn duplicate(&self) -> Self {
        Self {
            current_offset : self.current_offset,
            next_offset    : self.next_offset,
            range          : self.range.start..self.range.end,
            reversed       : self.reversed,
        }
    }

    fn duplicate_and_repeat(&self) -> Self {
        Self {
            current_offset : self.current_offset,
            next_offset    : self.current_offset,
            range          : self.range.start..self.range.end,
            reversed       : self.reversed,
        }
    }

    fn duplicate_and_advance(&self) -> Self {
        Self {
            current_offset : self.next_offset,
            next_offset    : self.next_offset,
            range          : self.range.start..self.range.end,
            reversed       : self.reversed,
        }
    }
}

impl From<PageTableEntryIndex> for PageTableEntryOffset {
    fn from(index:PageTableEntryIndex) -> Self {
        Self {
            current_offset : index.current_index * TABLE_OFFSET_INCREMENT_SIZE,
            next_offset    : index.next_index * TABLE_OFFSET_INCREMENT_SIZE,
            range          : (index.range.start * TABLE_OFFSET_INCREMENT_SIZE)..(index.range.end * TABLE_OFFSET_INCREMENT_SIZE),
            reversed       : index.reversed,
        }
    }
}
