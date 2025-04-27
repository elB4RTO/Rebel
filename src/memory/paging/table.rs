use crate::IntoOrPanic;
use crate::memory::{
    MemoryOwner, SIZE_8b,
};
use crate::memory::address::*;
use crate::memory::paging::*;
use crate::memory::paging::iterators::*;


/// Expresses the compatibility between entries
pub(in crate::memory)
enum Compatibility {
    Compatible,
    TooSmall,
    TooBig,
}


/// Defines the type of a [`PageTable`]
#[derive(Clone,Copy,PartialEq)]
pub(in crate::memory)
enum PageTableType {
    /*PageMapLevel5Table,*/ // TODO: add support for 5-levels paging
    PageMapLevel4Table,
    PageDirectoryPointerTable,
    PageDirectoryTable,
    PageTable,
}

impl PageTableType {
    /// Returns the parent [`PageTableType`] of a [`PageTableType`]
    ///
    /// ## Returns
    ///
    /// Returns an [`Err`] if the table cannot have parent tables
    pub(in crate::memory)
    fn parent_table_type(&self) -> Result<Self, PagingError> {
        use PageTableType::*;
        match self {
            PageMapLevel4Table => Err(PagingError::IncompatibleTable),
            PageDirectoryPointerTable => Ok(PageMapLevel4Table),
            PageDirectoryTable => Ok(PageDirectoryPointerTable),
            PageTable => Ok(PageDirectoryTable),
        }
    }

    /// Returns the parent [`PageTableType`] of a [`PageTableType`]
    ///
    /// ## Panics
    ///
    /// Panics if the table cannot have parent tables
    pub(in crate::memory)
    fn parent_table_type_or_panic(&self) -> Self {
        use PageTableType::*;
        match self {
            PageMapLevel4Table => PagingError::IncompatibleTable.panic(),
            PageDirectoryPointerTable => PageMapLevel4Table,
            PageDirectoryTable => PageDirectoryPointerTable,
            PageTable => PageDirectoryTable,
        }
    }

    /// Returns the child [`PageTableType`] of a [`PageTableType`]
    ///
    /// ## Returns
    ///
    /// Returns an [`Err`] if the table cannot have child tables
    pub(in crate::memory)
    fn child_table_type(&self) -> Result<Self, PagingError> {
        use PageTableType::*;
        match self {
            PageMapLevel4Table => Ok(PageDirectoryPointerTable),
            PageDirectoryPointerTable => Ok(PageDirectoryTable),
            PageDirectoryTable => Ok(PageTable),
            PageTable => Err(PagingError::IncompatibleTable),
        }
    }

    /// Returns the child [`PageTableType`] of a [`PageTableType`]
    ///
    /// ## Panics
    ///
    /// Panics if the table cannot have child tables
    pub(in crate::memory)
    fn child_table_type_or_panic(&self) -> Self {
        use PageTableType::*;
        match self {
            PageMapLevel4Table => PageDirectoryPointerTable,
            PageDirectoryPointerTable => PageDirectoryTable,
            PageDirectoryTable => PageTable,
            PageTable => PagingError::IncompatibleTable.panic(),
        }
    }
}
impl From<PageType> for PageTableType {
    fn from(child_page_type:PageType) -> Self {
        use PageTableType::*;
        match child_page_type {
            PageType::OneGiB  => PageDirectoryPointerTable,
            PageType::TwoMiB  => PageDirectoryTable,
            PageType::FourKiB => PageTable,
        }
    }
}


/// Represents an entry of a page table
#[derive(Clone)]
pub(in crate::memory)
struct PageTableEntry {
    /// The address of the entry in the parent table
    ///
    /// This is not the actual address of the entry in memory,
    /// it's the address of the parent table plus the offset of the entry.
    /// Dereferencing this address allows to access the bitmap of the entry,
    /// which stores its actual address and flags.
    pub(in crate::memory)
    entry : PhysicalAddress,
    /// The type of table this entry belongs to
    pub(in crate::memory)
    table_type : PageTableType,
    /// The owner of the table
    pub(in crate::memory)
    table_owner : MemoryOwner,
}

impl PageTableEntry {
    /// Creates a new [`PageTableEntry`] with the given data
    pub(in crate::memory)
    fn new(entry:PhysicalAddress, table_type:PageTableType, table_owner:MemoryOwner) -> Self {
        Self {
            entry,
            table_type,
            table_owner,
        }
    }

    /// Returns the [`PhysicalAddress`] of the entry in the parent table
    ///
    /// ## Note
    ///
    /// The returned address is not the actual address of the entry in memory,
    /// it's the address of the parent table plus the offset of the entry.
    /// Dereferencing this address allows to access the bitmap of the entry,
    /// which stores its actual address and flags.
    pub(in crate::memory)
    fn address_in_table(&self) -> PhysicalAddress {
        self.entry
    }

    /// Returns the [`Bitmap`] associated with the table entry
    ///
    /// ## Warning
    ///
    /// This function implies dereferencing the entry address
    /// in the table in order to get the bitmap it holds.
    /// Moreover, this function doesn't check whether the pointer
    /// is valid or whether the page/table is present.
    /// Use with extreme care.
    ///
    /// ## Example
    ///
    /// ```
    /// let next_table : u64 = 0x11FF8 | PRESENT | WRITABLE;
    /// unsafe { *(0x10000 as *mut u64) = next_table; }
    /// let entry = PageTable::from(0x10000).at_index(0.into()).unwrap();
    /// assert_eq!(entry.bitmap(), next_table);
    /// ```
    pub(in crate::memory)
    fn bitmap(&self) -> Bitmap {
        unsafe { Bitmap::from(self.entry.read::<u64>()) }
    }

    /// Applies the given [`Bitmap`] to the table entry
    ///
    /// ## Warning
    ///
    /// This function implies dereferencing the entry address
    /// in the table in order to set the bits.
    /// Moreover, this function doesn't check whether the pointer
    /// is valid or whether the page/table is present.
    /// Use with extreme care.
    ///
    /// ## Example
    ///
    /// ```
    /// let next_table : u64 = 0x11FF8 | PRESENT | WRITABLE;
    /// let entry = PageTable::from(0x10000).at_index(0.into()).unwrap();
    /// entry.set_bitmap(next_table.into());
    /// unsafe {
    ///   assert_eq!(*(0x10000 as *mut u64), next_table);
    /// }
    /// ```
    pub(in crate::memory)
    fn set_bitmap(&self, bitmap:Bitmap) {
        unsafe { self.entry.write::<u64>(bitmap.bits()) }
    }

    /// Checks the compatibility of this table entry with the given [`PageType`]
    ///
    /// ## Returns
    ///
    /// Returns [`Compatibility::TooBig`] if the given page type is too small to fit into
    /// the table to which the entry behold, [`Compatibility::Compatible`] if they're
    /// compatible, or [`Compatibility::TooSmall`] if the given page type is too big.
    ///
    /// ## Example
    ///
    /// ```
    /// let pml4t = PageTable::pml4t(MemoryOwner::User);
    /// let pml4t_entry = pml4t.at_offset_unchecked(0x0.into());
    /// assert_eq!(pml4t_entry.is_compatible(PageType::OneGib), Compatibility::TooBig);
    /// let pdpt = pml4t_entry.into().unwrap();
    /// let pdpt_entry = pdpt.at_offset_unchecked(0x0.into());
    /// assert_eq!(pdpt_entry.is_compatible(PageType::OneGib), Compatibility::Compatible);
    /// assert_eq!(pdpt_entry.is_compatible(PageType::TwoMib), Compatibility::TooBig);
    /// let pdt = pdpt_entry.into().unwrap();
    /// let pdt_entry = pdt.at_offset_unchecked(0x0.into());
    /// assert_eq!(pdt_entry.is_compatible(PageType::OneGib), Compatibility::TooSmall);
    /// assert_eq!(pdt_entry.is_compatible(PageType::TwoMib), Compatibility::Compatible);
    /// assert_eq!(pdt_entry.is_compatible(PageType::FourKib), Compatibility::TooBig);
    /// let pt = pdt_entry.into().unwrap();
    /// let pt_entry = pt.at_offset_unchecked(0x0.into());
    /// assert_eq!(pt_entry.is_compatible(PageType::TwoMib), Compatibility::TooSmall);
    /// assert_eq!(pt_entry.is_compatible(PageType::FourKib), Compatibility::Compatible);
    /// ```
    pub(in crate::memory)
    fn is_compatible(&self, page_type:PageType) -> Compatibility {
        use PageTableType::*;
        use PageType::*;
        match self.table_type {
            PageMapLevel4Table => Compatibility::TooBig,
            PageDirectoryPointerTable => match page_type {
                OneGiB => Compatibility::Compatible,
                _      => Compatibility::TooBig,
            },
            PageDirectoryTable => match page_type {
                OneGiB  => Compatibility::TooSmall,
                TwoMiB  => Compatibility::Compatible,
                FourKiB => Compatibility::TooBig,
            },
            PageTable => match page_type {
                FourKiB => Compatibility::Compatible,
                _       => Compatibility::TooSmall,
            },
        }
    }
}

impl IntoOrPanic<PageTable, PagingError> for PageTableEntry {
    fn into_or_panic(self) -> PageTable {
        PageTable {
            base_address : self.bitmap().address(PageType::FourKiB).into(),
            table_type   : self.table_type.child_table_type_or_panic(),
            table_owner  : self.table_owner,
        }
    }
}


/// Represents a page table
///
/// A [`PageTable`] can either be a `PML4T`, a `PDPT`, a `PDT` or a `PT`
#[derive(Clone)]
pub(in crate::memory)
struct PageTable {
    /// A pointer to the base table address
    pub(in crate::memory)
    base_address : PhysicalAddress,
    /// The type of table
    pub(in crate::memory)
    table_type : PageTableType,
    /// The owner of the table
    pub(in crate::memory)
    table_owner : MemoryOwner,
}

impl PageTable {
    /// Creates a new [`PageTable`] with the given data
    pub(in crate::memory)
    fn new(base_address:PhysicalAddress, table_type:PageTableType, table_owner:MemoryOwner) -> Self {
        Self {
            base_address,
            table_type,
            table_owner,
        }
    }

    /// Returns a [`PageTable`] representing the currently set `Page Map Level 4 Table`
    ///
    /// ## Note
    ///
    /// The address of the table is the one stored in the `CR3` register
    ///
    /// ## Panics
    ///
    /// Panics if the retrieved address is invalid (null)
    pub(in crate::memory)
    fn pml4t(table_owner:MemoryOwner) -> Self {
        match unsafe { crate::cpu::registers::get_cr3() } {
            0 => crate::panic("PageMapLevel4Table address is 0x0000000000000000"),
            table_addr => Self::new(table_addr.into(), PageTableType::PageMapLevel4Table, table_owner),
        }
    }

    /// Returns a `const` pointer to the physical address of the table
    /// plus the offset of the entry at the given index
    ///
    /// ## Note
    ///
    /// The returned address is not the address of the next page table or
    /// a page, it's the address inside the same table for the given index.
    ///
    /// ## Returns
    ///
    /// Returns [`None`] if `idx` is greater than the number of entries (512),
    /// otherwise returns the [`PageTableEntry`] at the given index.
    ///
    /// ## Example
    ///
    /// ```
    /// let table = PageTable::from(0x10000);
    /// assert_eq!(table.at_index(0.into()), Some(PageTableEntry::from(0x10000)));
    /// assert_eq!(table.at_index(511.into()), Some(PageTableEntry::from(0x10FF8)));
    /// assert_eq!(table.at_index(512.into()), None);
    /// ```
    pub(in crate::memory)
    fn at_index(&self, index:&PageTableEntryIndex) -> Option<PageTableEntry> {
        if let Some(idx) = index.get() {
            let entry_paddr = self.base_address + (idx * SIZE_8b);
            return Some(PageTableEntry::new(entry_paddr, self.table_type, self.table_owner));
        }
        None
    }

    /// Returns a `const` pointer to the physical address of the table
    /// plus the given offset
    ///
    /// ## Note
    ///
    /// The returned address is not the address of the next page table or
    /// a page, it's the address inside the same table at the given offset.
    ///
    /// ## Returns
    ///
    /// Returns [`None`] if `ofs` is greater than the maximum offset (0xFF8),
    /// otherwise returns the [`PageTableEntry`] at the given offset.
    ///
    /// ## Example
    ///
    /// ```
    /// let table = PageTable::from(0x10000);
    /// assert_eq!(table.at_offset(0x0.into()), Some(PageTableEntry::from(0x10000)));
    /// assert_eq!(table.at_offset(0xFF8.into()), Some(PageTableEntry::from(0x10FF8)));
    /// assert_eq!(table.at_offset(0x1000.into()), None);
    /// ```
    pub(in crate::memory)
    fn at_offset(&self, offset:&PageTableEntryOffset) -> Option<PageTableEntry> {
        if let Some(ofs) = offset.get() {
            let entry = self.base_address + ofs;
            return Some(PageTableEntry::new(entry, self.table_type, self.table_owner));
        }
        None
    }

    /// Like [`PageTable::at_offset()`] but doesn't check the bounds
    ///
    /// ## Warning
    ///
    /// Use with extreme caution.
    ///
    /// ## Returns
    ///
    /// Always returns the [`PageTableEntry`] at the given offset, even when invalid.
    ///
    /// ## Example
    ///
    /// ```
    /// let table = PageTable::from(0x10000);
    /// assert_eq!(table.at_offset_unchecked(0x0.into()), PageTableEntry::from(0x10000));
    /// assert_eq!(table.at_offset_unchecked(0xFF8.into()), PageTableEntry::from(0x10FF8));
    /// assert_eq!(table.at_offset_unchecked(0x1000.into()), PageTableEntry::from(0x11000));
    /// ```
    pub(in crate::memory)
    fn at_offset_unchecked(&self, offset:&PageTableEntryOffset) -> PageTableEntry {
        let entry = self.base_address + offset.get_unchecked();
        PageTableEntry::new(entry, self.table_type, self.table_owner)
    }

    /// Creates a [`PageTableIterator`] upon the table, to iterate its entries
    pub(in crate::memory)
    fn iterate_allocations(self) -> PageTableIterator {
        PageTableIterator::allocations(self)
    }

    /// Creates a [`PageTableRefIterator`] upon the table, to iterate its entries
    pub(in crate::memory)
    fn iterate_allocations_ref(&self) -> PageTableRefIterator {
        PageTableRefIterator::allocations(self)
    }

    /// Creates an [`UnboundedPageTableIterator`] upon the table, to iterate all of
    /// its entries
    ///
    /// ## Warning
    ///
    /// Use with caution, an [`UnboundedPageTableIterator`] should only be used in
    /// extermely rare cases since it will also iterate the reserved tables (such as
    /// the ones used to trace the memory allocations)
    pub(in crate::memory)
    fn iterate_unbounded(self) -> UnboundedPageTableIterator {
        UnboundedPageTableIterator::new(self)
    }

    /// Creates a [`PageTableEnumerator`] upon the table, to enumerate its entries
    pub(in crate::memory)
    fn enumerate_allocations(self) -> PageTableEnumerator {
        PageTableEnumerator::from(self.iterate_allocations())
    }

    /// Creates a [`PageTableRefEnumerator`] upon the table, to enumerate its entries
    pub(in crate::memory)
    fn enumerate_allocations_ref(&self) -> PageTableRefEnumerator {
        PageTableRefEnumerator::from(self.iterate_allocations_ref())
    }


    /// Creates an [`UnboundedPageTableEnumerator`] upon the table, to enumerate all of
    /// its entries
    ///
    /// ## Warning
    ///
    /// Use with caution, an [`UnboundedPageTableEnumerator`] should only be used in
    /// extermely rare cases since it will also enumerate the reserved tables (such as
    /// the ones used to trace the memory allocations)
    pub(in crate::memory)
    fn enumerate_unbounded(self) -> UnboundedPageTableEnumerator {
        UnboundedPageTableEnumerator::new(self)
    }

    /// Returns the first entry in the table that can be used for allocations and which
    /// is not marked as present
    ///
    /// ## Returns
    ///
    /// Returns [`Some`] if such an entry is found in the table, otherwise returns [`None`]
    pub(in crate::memory)
    fn find_available_allocation_entry(&self) -> Option<PageTableEntry> {
        for entry in self.iterate_allocations_ref() {
            if !entry.bitmap().present() {
                return Some(entry);
            }
        }
        None
    }
}

impl TryFrom<PageTableEntry> for PageTable {
    type Error = PagingError;
    /// Returns the [`PageTable`] pointed to by the given [`PageTableEntry`]
    ///
    /// ## Warning
    ///
    /// This function implies dereferencing the entry's pointer in order to get
    /// the address of the table it points to. Use with great care.
    ///
    /// ## Returns
    ///
    /// Returns an [`Err`] if the table's address is null (which usually means the entry
    /// has not been initialized, or it has been initialized improperly), if the entry's
    /// address is null, it the entry does not have the _Present_ bit, if it does have
    /// the _Page Size_ bit or if the entry belongs to a table that cannot have child
    /// tables (namely, a PT). Otherwise returns an [`Ok`] holding the table.
    fn try_from(entry:PageTableEntry) -> Result<Self, Self::Error> {
        if entry.address_in_table().is_null() {
            return Err(PagingError::AddressError(AddressError::NullAddress));
        }
        let entry_bitmap = entry.bitmap();
        if !entry_bitmap.present() {
            return Err(PagingError::TableNotPresent);
        } else if entry_bitmap.page_size() {
            return Err(PagingError::InvalidRequest);
        }
        let table_address = PhysicalAddress::from(entry_bitmap.address(PageType::FourKiB));
        if table_address.is_null() {
            return Err(PagingError::AddressError(AddressError::NullAddress));
        }
        Ok( Self {
            base_address : table_address,
            table_type   : entry.table_type.child_table_type()?,
            table_owner  : entry.table_owner,
        })
    }
}
