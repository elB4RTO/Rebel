use crate::LogError;
use crate::memory::MemoryOwner;
use crate::memory::address::*;
use crate::memory::paging::*;
use crate::memory::paging::iterators::*;


/// Represents a physical address
#[repr(align(8))]
#[derive(Clone,Copy,PartialEq,PartialOrd)]
pub(crate)
struct PhysicalAddress {
    address : u64,
}

impl PhysicalAddress {
    /// Converts the address from physical to logical
    ///
    /// ## Warning
    ///
    /// This function is quite expensive, its usage should be limited to very few
    /// edge cases in which the logical address cannot be (indirectly) retrieved
    /// in any other way.
    ///
    /// ## Returns
    ///
    /// Returns an [`Err`] if no page is found that contains the address or in case of
    /// serious internal failure, otherwise returns an [`Ok`] with the logical address.
    pub(in crate::memory)
    fn to_logical(&self, memory_owner:MemoryOwner) -> AddressResult<LogicalAddress> {
        let pml4t = PageTable::pml4t(memory_owner);
        for (pml4t_idx,pml4t_entry) in pml4t.enumerate_allocations() {
            if !pml4t_entry.bitmap().present() {
                continue;
            }
            let pdpt = PageTable::try_from(pml4t_entry)
                .log_map_err(AddressError::LogicalToPhysical)?;
            for (pdpt_idx,pdpt_entry) in pdpt.enumerate_allocations() {
                let pdpt_entry_bits = pdpt_entry.bitmap();
                if !pdpt_entry_bits.present() {
                    continue;
                } else if pdpt_entry_bits.page_size() {
                    let address = self.page_address(PageType::OneGiB);
                    let entry_address = pdpt_entry_bits.address(PageType::OneGiB);
                    if entry_address == address {
                        let offset = self.page_offset(PageType::OneGiB);
                        let laddr = LogicalAddress::from(offset)
                            .with_pml4t_index(pml4t_idx)
                            .with_pdpt_index(pdpt_idx)
                            .as_canonical();
                        return Ok(laddr);
                    }
                    continue;
                }
                let pdt = PageTable::try_from(pdpt_entry)
                    .log_map_err(AddressError::LogicalToPhysical)?;
                for (pdt_idx,pdt_entry) in pdt.enumerate_allocations() {
                    let pdt_entry_bits = pdt_entry.bitmap();
                    if !pdt_entry_bits.present() {
                        continue;
                    } else if pdt_entry_bits.page_size() {
                        let address = self.page_address(PageType::TwoMiB);
                        let entry_address = pdt_entry_bits.address(PageType::TwoMiB);
                        if entry_address == address {
                            let offset = self.page_offset(PageType::TwoMiB);
                            let laddr = LogicalAddress::from(offset)
                                .with_pml4t_index(pml4t_idx)
                                .with_pdpt_index(pdpt_idx)
                                .with_pdt_index(pdt_idx)
                                .as_canonical();
                            return Ok(laddr);
                        }
                        continue;
                    }
                    let pt = PageTable::try_from(pdt_entry)
                        .log_map_err(AddressError::LogicalToPhysical)?;
                    for (pt_idx,pt_entry) in pt.enumerate_allocations() {
                        let pt_entry_bits = pt_entry.bitmap();
                        if !pt_entry_bits.present() {
                            continue;
                        }
                        let address = self.page_address(PageType::FourKiB);
                        let entry_address = pt_entry_bits.address(PageType::FourKiB);
                        if entry_address == address {
                            let offset = self.page_offset(PageType::FourKiB);
                            let laddr = LogicalAddress::from(offset)
                                .with_pml4t_index(pml4t_idx)
                                .with_pdpt_index(pdpt_idx)
                                .with_pdt_index(pdt_idx)
                                .with_pt_index(pt_idx)
                                .as_canonical();
                            return Ok(laddr);
                        }
                    }
                }
            }
        }
        Err(AddressError::PhysicalToLogical)
    }

    /// As [`Self::to_logical()`], but not restricted to the allocation tables only
    pub(in crate::memory)
    fn to_logical_unbounded(&self) -> AddressResult<LogicalAddress> {
        let pml4t = PageTable::pml4t(MemoryOwner::User);
        for (pml4t_idx,pml4t_entry) in pml4t.enumerate_unbounded() {
            if !pml4t_entry.bitmap().present() {
                continue;
            }
            let pdpt = PageTable::try_from(pml4t_entry)
                .log_map_err(AddressError::LogicalToPhysical)?;
            for (pdpt_idx,pdpt_entry) in pdpt.enumerate_unbounded() {
                let pdpt_entry_bits = pdpt_entry.bitmap();
                if !pdpt_entry_bits.present() {
                    continue;
                } else if pdpt_entry_bits.page_size() {
                    let address = self.page_address(PageType::OneGiB);
                    let entry_address = pdpt_entry_bits.address(PageType::OneGiB);
                    if entry_address == address {
                        let offset = self.page_offset(PageType::OneGiB);
                        let laddr = LogicalAddress::from(offset)
                            .with_pml4t_index(pml4t_idx)
                            .with_pdpt_index(pdpt_idx)
                            .as_canonical();
                        return Ok(laddr);
                    }
                }
                let pdt = PageTable::try_from(pdpt_entry)
                    .log_map_err(AddressError::LogicalToPhysical)?;
                for (pdt_idx,pdt_entry) in pdt.enumerate_unbounded() {
                    let pdt_entry_bits = pdt_entry.bitmap();
                    if !pdt_entry_bits.present() {
                        continue;
                    } else if pdt_entry_bits.page_size() {
                        let address = self.page_address(PageType::TwoMiB);
                        let entry_address = pdt_entry_bits.address(PageType::TwoMiB);
                        if entry_address == address {
                            let offset = self.page_offset(PageType::TwoMiB);
                            let laddr = LogicalAddress::from(offset)
                                .with_pml4t_index(pml4t_idx)
                                .with_pdpt_index(pdpt_idx)
                                .with_pdt_index(pdt_idx)
                                .as_canonical();
                            return Ok(laddr);
                        }
                    }
                    let pt = PageTable::try_from(pdt_entry)
                        .log_map_err(AddressError::LogicalToPhysical)?;
                    for (pt_idx,pt_entry) in pt.enumerate_unbounded() {
                        let pt_entry_bits = pt_entry.bitmap();
                        if !pt_entry_bits.present() {
                            continue;
                        }
                        let address = self.page_address(PageType::FourKiB);
                        let entry_address = pt_entry_bits.address(PageType::FourKiB);
                        if entry_address == address {
                            let offset = self.page_offset(PageType::FourKiB);
                            let laddr = LogicalAddress::from(offset)
                                .with_pml4t_index(pml4t_idx)
                                .with_pdpt_index(pdpt_idx)
                                .with_pdt_index(pdt_idx)
                                .with_pt_index(pt_idx)
                                .as_canonical();
                            return Ok(laddr);
                        }
                    }
                }
            }
        }
        Err(AddressError::PhysicalToLogical)
    }

    /// Returns the part of the address which represents the base address of the
    /// page it belongs to
    ///
    /// ## Example
    ///
    /// ```
    /// // assuming 40 bits address size
    /// let paddr = PhysicalAddress::from(0x7007007AAA);
    /// assert_eq!(paddr.get(), 0b0111000000_000111000_000000111_101010101010);
    /// assert_eq!(paddr.page_address(PageType::OneGiB), 0b0111000000);
    /// assert_eq!(paddr.page_address(PageType::TwoMiB), 0b0111000000_000111000);
    /// assert_eq!(paddr.page_address(PageType::FourKiB), 0b0111000000_000111000_000000111);
    /// ```
    fn page_address(&self, page_type:PageType) -> u64 {
        match page_type {
            PageType::OneGiB  => unsafe { self.address & BITMASK_ADDRESS_1G },
            PageType::TwoMiB  => unsafe { self.address & BITMASK_ADDRESS_2M },
            PageType::FourKiB => unsafe { self.address & BITMASK_ADDRESS_4K },
        }
    }

    /// Returns the part of the address which represents the offset from the base
    /// address of the page it belongs to
    ///
    /// ## Example
    ///
    /// ```
    /// // assuming 40 bits address size
    /// let paddr = PhysicalAddress::from(0x7007007AAA);
    /// assert_eq!(paddr.get(), 0b0111000000_000111000_000000111_101010101010);
    /// assert_eq!(paddr.page_offset(PageType::OneGiB), 0b000111000_000000111_101010101010);
    /// assert_eq!(paddr.page_offset(PageType::TwoMiB), 0b000000111_101010101010);
    /// assert_eq!(paddr.page_offset(PageType::FourKiB), 0b101010101010);
    /// ```
    fn page_offset(&self, page_type:PageType) -> u64 {
        match page_type {
            PageType::OneGiB  => self.address & BITMASK_OFFSET_1G,
            PageType::TwoMiB  => self.address & BITMASK_OFFSET_2M,
            PageType::FourKiB => self.address & BITMASK_OFFSET_4K,
        }
    }
}

impl Address for PhysicalAddress {
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

    unsafe fn as_ref<T>(&self) -> &T {
        &*self.as_ptr::<T>()
    }

    unsafe fn as_ref_mut<T>(&self) -> &mut T {
        &mut*self.as_ptr_mut::<T>()
    }

    unsafe fn read<T>(&self) -> T {
        core::ptr::read(self.as_ptr::<T>())
    }

    unsafe fn write<T>(&self, value:T) {
        core::ptr::write(self.as_ptr_mut::<T>(), value)
    }
}

impl Align<u64> for PhysicalAddress {
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

impl Align<PageType> for PhysicalAddress {
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

impl From<u64> for PhysicalAddress {
    fn from(value:u64) -> Self {
        Self {
            address : value,
        }
    }
}

impl Add<u64> for PhysicalAddress {
    type Output = PhysicalAddress;
    /// Adds the given offset from the address
    ///
    /// ## Example
    ///
    /// ```
    /// let paddr = PhysicalAddress::from(0x10000);
    /// assert_eq!(paddr.add(0xFFF), PhysicalAddress::from(0x10FFF));
    /// let paddr = PhysicalAddress::from(0x10FFF);
    /// assert_eq!(paddr.add(0x1), PhysicalAddress::from(0x11000));
    /// ```
    fn add(self, offset:u64) -> Self {
        Self::from(self.address + offset)
    }
}

impl Add<PageTableEntryOffset> for PhysicalAddress {
    type Output = PhysicalAddress;
    /// Adds the given offset from the address
    ///
    /// ## Example
    ///
    /// ```
    /// let paddr = PhysicalAddress::from(0x10000);
    /// assert_eq!(paddr.add(0xFF8.into()), PhysicalAddress::from(0x10FF8));
    /// ```
    fn add(self, offset:PageTableEntryOffset) -> Self {
        Self::from(self.address + offset.get_unchecked())
    }
}

impl AddAssign<u64> for PhysicalAddress {
    /// Adds the given offset from the address
    ///
    /// ## Example
    ///
    /// ```
    /// let mut paddr = PhysicalAddress::from(0x10000);
    /// paddr.add_assign(0xFFF);
    /// assert_eq!(paddr, PhysicalAddress::from(0x10FFF));
    /// let mut paddr = PhysicalAddress::from(0x10FFF);
    /// paddr.add_assign(0x1);
    /// assert_eq!(paddr, PhysicalAddress::from(0x11000));
    /// ```
    fn add_assign(&mut self, offset:u64) {
        self.address += offset;
    }
}

impl Sub<u64> for PhysicalAddress {
    type Output = PhysicalAddress;
    /// Subtracts the given offset from the address
    ///
    /// ## Example
    ///
    /// ```
    /// let paddr = PhysicalAddress::from(0x10FFF);
    /// assert_eq!(paddr.sub(0xFFF), PhysicalAddress::from(0x10000));
    /// let paddr = PhysicalAddress::from(0x11000);
    /// assert_eq!(paddr.sub(0x1), PhysicalAddress::from(0x10FFF));
    /// ```
    fn sub(self, offset:u64) -> Self {
        Self::from(self.address - offset)
    }
}

impl Sub<PhysicalAddress> for PhysicalAddress {
    type Output = u64;
    /// Subtracts the given address from the address
    ///
    /// ## Example
    ///
    /// ```
    /// let paddr1 = PhysicalAddress::from(0x10FFF);
    /// let paddr2 = PhysicalAddress::from(0x10000);
    /// let offset = paddr1.sub(paddr2);
    /// assert_eq!(offset, 0xFFF);
    /// ```
    fn sub(self, other:PhysicalAddress) -> Self::Output {
        self.address - other.address
    }
}

impl SubAssign<u64> for PhysicalAddress {
    /// Subtracts the given offset from the address
    ///
    /// ## Example
    ///
    /// ```
    /// let mut paddr = PhysicalAddress::from(0x10FFF);
    /// paddr.sub_assign(0xFFF);
    /// assert_eq!(paddr, PhysicalAddress::from(0x10000));
    /// let mut paddr = PhysicalAddress::from(0x11000);
    /// paddr.sub_assign(0x1);
    /// assert_eq!(paddr, PhysicalAddress::from(0x10FFF));
    /// ```
    fn sub_assign(&mut self, offset:u64) {
        self.address -= offset;
    }
}
