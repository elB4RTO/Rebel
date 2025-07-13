
// TRANSLATION OF VIRTUAL ADDRESSES
//   # # # # # # # # # # # # # # # # # # # # # # # # # # # # # # # # # # # # # # # # # # # # # # # # # # # # # # # # # # # # # # # #
//
// 1 GiB PAGE
//  ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┯━━━━━━━━━━━━━━━━━┯━━━━━━━━━━━━━━━━━┯━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
//  ┃ 63                         48 │ 47           39 │ 38           30 │ 29                                                      0 ┃
//  ┠-------------------------------┿-----------------┿-----------------┿-----------------------------------------------------------┨
//  ┃             unused            │      PML4       │      PDPT       │                          offset                           ┃
//  ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┷━━━━━━━━━━━━━━━━━┷━━━━━━━━━━━━━━━━━┷━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛
//
// 2 MiB PAGE
//  ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┯━━━━━━━━━━━━━━━━━┯━━━━━━━━━━━━━━━━━┯━━━━━━━━━━━━━━━━━━━┯━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
//  ┃ 63                         48 │ 47           39 │ 38           30 │ 29             21 │ 20                                  0 ┃
//  ┠-------------------------------┿-----------------┿-----------------┿-------------------┿---------------------------------------┨
//  ┃             unused            │      PML4       │      PDPT       │        PDT        │                offset                 ┃
//  ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┷━━━━━━━━━━━━━━━━━┷━━━━━━━━━━━━━━━━━┷━━━━━━━━━━━━━━━━━━━┷━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛
//
// 4 KiB PAGE
//  ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┯━━━━━━━━━━━━━━━━━┯━━━━━━━━━━━━━━━━━┯━━━━━━━━━━━━━━━━━━━┯━━━━━━━━━━━━━━━━━┯━━━━━━━━━━━━━━━━━━━━━┓
//  ┃ 63                         48 │ 47           39 │ 38           30 │ 29             21 │ 20           12 │ 11                0 ┃
//  ┠-------------------------------┿-----------------┿-----------------┿-------------------┿-----------------┿---------------------┨
//  ┃             unused            │      PML4       │      PDPT       │        PDT        │       PT        │       offset        ┃
//  ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┷━━━━━━━━━━━━━━━━━┷━━━━━━━━━━━━━━━━━┷━━━━━━━━━━━━━━━━━━━┷━━━━━━━━━━━━━━━━━┷━━━━━━━━━━━━━━━━━━━━━┛


use crate::LogError;
use crate::memory::MemoryOwner;
use crate::memory::address::*;
use crate::memory::paging::*;
use crate::memory::paging::iterators::*;


const CANONICAL_BIT         : u64 = 1 << 47;
const HIGHER_16_BITS        : u64 = 0b1111111111111111000000000000000000000000000000000000000000000000;

const PML4T_SHIFT           : u64 = 39;
const PDPT_SHIFT            : u64 = 30;
const PDT_SHIFT             : u64 = 21;
const PT_SHIFT              : u64 = 12;

const TABLE_INDEX_BITMASK   : u64 = 0x1FF;


/// Represents a logical address
#[repr(align(8))]
#[derive(Clone,Copy,PartialEq,PartialOrd)]
pub(crate)
struct LogicalAddress {
    address : u64,
}

impl LogicalAddress {
    /// Converts the address from logical to physical
    ///
    /// ## Returns
    ///
    /// Returns an [`Err`] if any of the pages composing the address cannot be found
    /// or in case of serious internal failure, otherwise returns an [`Ok`] with the
    /// physical address.
    pub(in crate::memory)
    fn to_physical(&self, memory_owner:MemoryOwner) -> AddressResult<PhysicalAddress> {
        let pml4t = PageTable::pml4t(memory_owner);
        let pml4t_idx = PageTableEntryIndex::allocations_with_index(self.pml4t_index(), PageTableType::PageMapLevel4Table, memory_owner);
        if let Some(pml4t_entry) = pml4t.at_index(&pml4t_idx) {
            let pml4t_entry_bits = pml4t_entry.bitmap();
            if !pml4t_entry_bits.present() {
                return Err(PagingError::PageNotPresent)
                    .log_map_err(AddressError::LogicalToPhysical);
            }
            let pdpt = PageTable::try_from(pml4t_entry)
                .log_map_err(AddressError::LogicalToPhysical)?;
            let pdpt_idx = PageTableEntryIndex::allocations_with_index(self.pdpt_index(), PageTableType::PageDirectoryPointerTable, memory_owner);
            if let Some(pdpt_entry) = pdpt.at_index(&pdpt_idx) {
                let pdpt_entry_bits = pdpt_entry.bitmap();
                if !pdpt_entry_bits.present() {
                    return Err(PagingError::PageNotPresent)
                        .log_map_err(AddressError::LogicalToPhysical);
                } else if pdpt_entry_bits.page_size() {
                    let offset = self.page_offset(PageType::OneGiB);
                    let address = pdpt_entry_bits.address(PageType::OneGiB);
                    return Ok((address|offset).into());
                }
                let pdt = PageTable::try_from(pdpt_entry)
                    .log_map_err(AddressError::LogicalToPhysical)?;
                let pdt_idx = PageTableEntryIndex::allocations_with_index(self.pdt_index(), PageTableType::PageDirectoryTable, memory_owner);
                if let Some(pdt_entry) = pdt.at_index(&pdt_idx) {
                    let pdt_entry_bits = pdt_entry.bitmap();
                    if !pdt_entry_bits.present() {
                        return Err(PagingError::PageNotPresent)
                            .log_map_err(AddressError::LogicalToPhysical);
                    } else if pdt_entry_bits.page_size() {
                        let offset = self.page_offset(PageType::TwoMiB);
                        let address = pdt_entry_bits.address(PageType::TwoMiB);
                        return Ok((address|offset).into());
                    }
                    let pt = PageTable::try_from(pdt_entry)
                        .log_map_err(AddressError::LogicalToPhysical)?;
                    let pt_idx = PageTableEntryIndex::allocations_with_index(self.pt_index(), PageTableType::PageTable, memory_owner);
                    if let Some(pt_entry) = pt.at_index(&pt_idx) {
                        let pt_entry_bits = pt_entry.bitmap();
                        if !pt_entry_bits.present() {
                            return Err(PagingError::PageNotPresent)
                                .log_map_err(AddressError::LogicalToPhysical);
                        }
                        let offset = self.page_offset(PageType::FourKiB);
                        let address = pt_entry_bits.address(PageType::FourKiB);
                        return Ok((address|offset).into());
                    }
                }
            }
        }
        Err(PagingError::OutOfBounds)
            .log_map_err(AddressError::LogicalToPhysical)
    }

    /// Same as [`Self::to_physical()`], but not restricted to the allocation tables only
    pub(in crate::memory)
    fn to_physical_unbounded(&self) -> AddressResult<PhysicalAddress> {
        let pml4t = PageTable::pml4t(MemoryOwner::User);
        let pml4t_idx = PageTableEntryIndex::unbounded_with_index(self.pml4t_index());
        if let Some(pml4t_entry) = pml4t.at_index(&pml4t_idx) {
            let pml4t_entry_bits = pml4t_entry.bitmap();
            if !pml4t_entry_bits.present() {
                return Err(PagingError::PageNotPresent)
                    .log_map_err(AddressError::LogicalToPhysical);
            }
            let pdpt = PageTable::try_from(pml4t_entry)
                .log_map_err(AddressError::LogicalToPhysical)?;
            let pdpt_idx = PageTableEntryIndex::unbounded_with_index(self.pdpt_index());
            if let Some(pdpt_entry) = pdpt.at_index(&pdpt_idx) {
                let pdpt_entry_bits = pdpt_entry.bitmap();
                if !pdpt_entry_bits.present() {
                    return Err(PagingError::PageNotPresent)
                        .log_map_err(AddressError::LogicalToPhysical);
                } else if pdpt_entry_bits.page_size() {
                    let offset = self.page_offset(PageType::OneGiB);
                    let address = pdpt_entry_bits.address(PageType::OneGiB);
                    return Ok((address|offset).into());
                }
                let pdt = PageTable::try_from(pdpt_entry)
                    .log_map_err(AddressError::LogicalToPhysical)?;
                let pdt_idx = PageTableEntryIndex::unbounded_with_index(self.pdt_index());
                if let Some(pdt_entry) = pdt.at_index(&pdt_idx) {
                    let pdt_entry_bits = pdt_entry.bitmap();
                    if !pdt_entry_bits.present() {
                        return Err(PagingError::PageNotPresent)
                            .log_map_err(AddressError::LogicalToPhysical);
                    } else if pdt_entry_bits.page_size() {
                        let offset = self.page_offset(PageType::TwoMiB);
                        let address = pdt_entry_bits.address(PageType::TwoMiB);
                        return Ok((address|offset).into());
                    }
                    let pt = PageTable::try_from(pdt_entry)
                        .log_map_err(AddressError::LogicalToPhysical)?;
                    let pt_idx = PageTableEntryIndex::unbounded_with_index(self.pt_index());
                    if let Some(pt_entry) = pt.at_index(&pt_idx) {
                        let pt_entry_bits = pt_entry.bitmap();
                        if !pt_entry_bits.present() {
                            return Err(PagingError::PageNotPresent)
                                .log_map_err(AddressError::LogicalToPhysical);
                        }
                        let offset = self.page_offset(PageType::FourKiB);
                        let address = pt_entry_bits.address(PageType::FourKiB);
                        return Ok((address|offset).into());
                    }
                }
            }
        }
        Err(PagingError::OutOfBounds)
            .log_map_err(AddressError::LogicalToPhysical)
    }

    /// Returns the part of the address which represents the index in the PML4T table
    ///
    /// ## Example
    ///
    /// ```
    /// let laddr = LogicalAddress::from(0x0000008080604070);
    /// assert_eq!(laddr.get(), 0b0000000000000000_000000001_000000010_000000011_000000100_000001110000);
    /// assert_eq!(laddr.mpl4t_index(), 0b000000001);
    /// ```
    pub(in crate::memory) const
    fn pml4t_index(&self) -> u64 {
        (self.address >> PML4T_SHIFT) & TABLE_INDEX_BITMASK
    }

    /// Returns the part of the address which represents the index in the PDPT table
    ///
    /// ## Example
    ///
    /// ```
    /// let laddr = LogicalAddress::from(0x0000008080604070);
    /// assert_eq!(laddr.get(), 0b0000000000000000_000000001_000000010_000000011_000000100_000001110000);
    /// assert_eq!(laddr.mpl4t_index(), 0b000000010);
    /// ```
    pub(in crate::memory) const
    fn pdpt_index(&self) -> u64 {
        (self.address >> PDPT_SHIFT) & TABLE_INDEX_BITMASK
    }

    /// Returns the part of the address which represents the index in the PDT table
    ///
    /// ## Example
    ///
    /// ```
    /// let laddr = LogicalAddress::from(0x0000008080604070);
    /// assert_eq!(laddr.get(), 0b0000000000000000_000000001_000000010_000000011_000000100_000001110000);
    /// assert_eq!(laddr.mpl4t_index(), 0b000000011);
    /// ```
    pub(in crate::memory) const
    fn pdt_index(&self) -> u64 {
        (self.address >> PDT_SHIFT) & TABLE_INDEX_BITMASK
    }

    /// Returns the part of the address which represents the index in the PT table
    ///
    /// ## Example
    ///
    /// ```
    /// let laddr = LogicalAddress::from(0x0000008080604070);
    /// assert_eq!(laddr.get(), 0b0000000000000000_000000001_000000010_000000011_000000100_000001110000);
    /// assert_eq!(laddr.mpl4t_index(), 0b000000100);
    /// ```
    pub(in crate::memory) const
    fn pt_index(&self) -> u64 {
        (self.address >> PT_SHIFT) & TABLE_INDEX_BITMASK
    }

    /// Returns the part of the address which represents the offset in the page
    ///
    /// ## Example
    ///
    /// ```
    /// let laddr = LogicalAddress::from(0x0000008080604070);
    /// assert_eq!(laddr.get(), 0b0000000000000000_000000001_000000010_000000011_000000100_000001110000);
    /// assert_eq!(laddr.mpl4t_index(), 0b000001110000);
    /// ```
    pub(in crate::memory) const
    fn page_offset(&self, page_type:PageType) -> u64 {
        match page_type {
            PageType::OneGiB  => self.address & BITMASK_OFFSET_1G,
            PageType::TwoMiB  => self.address & BITMASK_OFFSET_2M,
            PageType::FourKiB => self.address & BITMASK_OFFSET_4K,
        }
    }

    /// Sets the PML4T index part of the address to the given index
    ///
    /// ## Warning
    ///
    /// This function does not clean the old bits before adding the new ones
    /// and thus it shall only be used during the creation of the address and
    /// *never* against an already existing one
    ///
    /// ## Example
    ///
    /// ```
    /// let laddr = LogicalAddress::from(0x0000000000000000).with_mpl4t_index(0b000000001);
    /// assert_eq!(laddr.get(), 0b0000000000000000_000000001_000000000_000000000_000000000_000000000000);
    /// ```
    pub(in crate::memory) const
    fn with_pml4t_index(mut self, idx:u64) -> Self {
        self.address |= idx << PML4T_SHIFT;
        self
    }

    /// Sets the PDPT index part of the address to the given index
    ///
    /// ## Warning
    ///
    /// This function does not clean the old bits before adding the new ones
    /// and thus it shall only be used during the creation of the address and
    /// *never* against an already existing one
    ///
    /// ## Example
    ///
    /// ```
    /// let laddr = LogicalAddress::from(0x0000000000000000).with_pdpt_index(0b000000010);
    /// assert_eq!(laddr.get(), 0b0000000000000000_000000000_000000010_000000000_000000000_000000000000);
    /// ```
    pub(in crate::memory) const
    fn with_pdpt_index(mut self, idx:u64) -> Self {
        self.address |= idx << PDPT_SHIFT;
        self
    }

    /// Sets the PDT index part of the address to the given index
    ///
    /// ## Warning
    ///
    /// This function does not clean the old bits before adding the new ones
    /// and thus it shall only be used during the creation of the address and
    /// *never* against an already existing one
    ///
    /// ## Example
    ///
    /// ```
    /// let laddr = LogicalAddress::from(0x0000000000000000).with_pdt_index(0b000000011);
    /// assert_eq!(laddr.get(), 0b0000000000000000_000000000_000000000_000000011_000000000_000000000000);
    /// ```
    pub(in crate::memory) const
    fn with_pdt_index(mut self, idx:u64) -> Self {
        self.address |= idx << PDT_SHIFT;
        self
    }

    /// Sets the PT index part of the address to the given index
    ///
    /// ## Warning
    ///
    /// This function does not clean the old bits before adding the new ones
    /// and thus it shall only be used during the creation of the address and
    /// *never* against an already existing one
    ///
    /// ## Example
    ///
    /// ```
    /// let laddr = LogicalAddress::from(0x0000000000000000).with_pt_index(0b000000100);
    /// assert_eq!(laddr.get(), 0b0000000000000000_000000000_000000000_000000000_000000100_000000000000);
    /// ```
    pub(in crate::memory) const
    fn with_pt_index(mut self, idx:u64) -> Self {
        self.address |= idx << PT_SHIFT;
        self
    }

    /// Sets the page offset part of the address to the given offset
    ///
    /// ## Warning
    ///
    /// This function does not clean the old bits before adding the new ones
    /// and thus it shall only be used during the creation of the address and
    /// *never* against an already existing one
    ///
    /// ## Example
    ///
    /// ```
    /// let laddr = LogicalAddress::from(0x0000000000000000).with_page_offset(0b000001110000);
    /// assert_eq!(laddr.get(), 0b0000000000000000_000000000_000000000_000000000_000000000_000001110000);
    /// ```
    pub(in crate::memory) const
    fn with_page_offset(mut self, ofs:u64) -> Self {
        self.address |= ofs;
        self
    }

    /// Canonicalizes the address
    ///
    /// ## Warning
    ///
    /// This function does not clean the old bits before adding the new ones
    /// and thus it shall only be used during the creation of the address and
    /// *never* against an already existing one
    ///
    /// ## Example
    ///
    /// ```
    /// let laddr1 = LogicalAddress::from(0x0000000000000000).as_canonical();
    /// assert_eq!(laddr1.get(), 0b0000000000000000_000000000_000000000_000000000_000000000_000000000000);
    ///
    /// let laddr2 = LogicalAddress::from(0x0000800000000000).as_canonical();
    /// assert_eq!(laddr2.get(), 0b1111111111111111_100000000_000000000_000000000_000000000_000000000000);
    /// ```
    pub(in crate::memory) const
    fn as_canonical(mut self) -> Self {
        if (self.address | CANONICAL_BIT) == CANONICAL_BIT {
            self.address |= HIGHER_16_BITS;
        }
        self
    }
}

impl Address for LogicalAddress {
    fn is_null(&self) -> bool {
        self.address == 0
    }

    fn get(&self) -> u64 {
        self.address
    }

    fn as_ptr<T>(&self) -> *const T {
        self.address as *const T
    }

    fn as_ptr_mut<T>(&self) -> *mut T {
        self.address as *mut T
    }
}

impl Align<u64> for LogicalAddress {
    fn is_aligned(&self, bound:u64) -> bool {
        self.address % bound == 0
    }

    fn align_to_lower(&mut self, bound:u64) {
        self.address -= self.address % bound;
    }

    fn aligned_to_lower(mut self, bound:u64) -> Self {
        self.align_to_lower(bound);
        self
    }

    fn align_to_upper(&mut self, bound:u64) {
        let unalignment = self.address % bound;
        if unalignment > 0 {
            self.address += bound - unalignment;
        }
    }

    fn aligned_to_upper(mut self, bound:u64) -> Self {
        self.align_to_upper(bound);
        self
    }

    fn force_align_to_upper(&mut self, bound:u64) {
        self.address += bound - (self.address % bound);
    }

    fn force_aligned_to_upper(mut self, bound:u64) -> Self {
        self.force_align_to_upper(bound);
        self
    }
}

impl Align<PageType> for LogicalAddress {
    fn is_aligned(&self, bound:PageType) -> bool {
        <Self as Align<u64>>::is_aligned(self, bound.into())
    }

    fn align_to_lower(&mut self, bound:PageType) {
        <Self as Align<u64>>::align_to_lower(self, bound.into())
    }

    fn aligned_to_lower(self, bound:PageType) -> Self {
        <Self as Align<u64>>::aligned_to_lower(self, bound.into())
    }

    fn align_to_upper(&mut self, bound:PageType) {
        <Self as Align<u64>>::align_to_upper(self, bound.into())
    }

    fn aligned_to_upper(self, bound:PageType) -> Self {
        <Self as Align<u64>>::aligned_to_upper(self, bound.into())
    }

    fn force_align_to_upper(&mut self, bound:PageType) {
        <Self as Align<u64>>::force_align_to_upper(self, bound.into())
    }

    fn force_aligned_to_upper(self, bound:PageType) -> Self {
        <Self as Align<u64>>::force_aligned_to_upper(self, bound.into())
    }
}

impl From<u64> for LogicalAddress {
    fn from(value:u64) -> Self {
        Self {
            address : value,
        }
    }
}

impl Add<u64> for LogicalAddress {
    type Output = LogicalAddress;
    /// Adds the given offset to the address
    ///
    /// ## Example
    ///
    /// ```
    /// let laddr = LogicalAddress::from(0x10000);
    /// assert_eq!(laddr.add(0xFFF), LogicalAddress::from(0x10FFF));
    /// let laddr = LogicalAddress::from(0x10FFF);
    /// assert_eq!(laddr.add(0x1), LogicalAddress::from(0x11000));
    /// ```
    fn add(self, offset:u64) -> Self {
        Self::from(self.address + offset)
    }
}

impl AddAssign<u64> for LogicalAddress {
    /// Adds the given offset from the address
    ///
    /// ## Example
    ///
    /// ```
    /// let mut laddr = LogicalAddress::from(0x10000);
    /// laddr.add_assign(0xFFF);
    /// assert_eq!(laddr, LogicalAddress::from(0x10FFF));
    /// let mut laddr = LogicalAddress::from(0x10FFF);
    /// laddr.add_assign(0x1);
    /// assert_eq!(laddr, LogicalAddress::from(0x11000));
    /// ```
    fn add_assign(&mut self, offset:u64) {
        self.address += offset;
    }
}

impl Sub<u64> for LogicalAddress {
    type Output = LogicalAddress;
    /// Subtracts the given offset from the address
    ///
    /// ## Example
    ///
    /// ```
    /// let laddr = LogicalAddress::from(0x10FFF);
    /// assert_eq!(laddr.sub(0xFFF), LogicalAddress::from(0x10000));
    /// let laddr = LogicalAddress::from(0x11000);
    /// assert_eq!(laddr.sub(0x1), LogicalAddress::from(0x10FFF));
    /// ```
    fn sub(self, offset:u64) -> Self {
        Self::from(self.address - offset)
    }
}

impl SubAssign<u64> for LogicalAddress {
    /// Subtracts the given offset from the address
    ///
    /// ## Example
    ///
    /// ```
    /// let mut laddr = LogicalAddress::from(0x10FFF);
    /// laddr.sub_assign(0xFFF);
    /// assert_eq!(laddr, LogicalAddress::from(0x10000));
    /// let mut laddr = LogicalAddress::from(0x11000);
    /// laddr.sub_assign(0x1);
    /// assert_eq!(laddr, LogicalAddress::from(0x10FFF));
    /// ```
    fn sub_assign(&mut self, offset:u64) {
        self.address -= offset;
    }
}
