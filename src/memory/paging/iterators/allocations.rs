use crate::LogError;
use crate::memory::address::LogicalAddress;
use crate::memory::paging::*;
use crate::memory::paging::iterators::*;

use core::ops::Add;


/// Wrapper for the offsets of an [`AllocationsIterator`]
pub(in crate::memory::paging)
enum AllocationsIteratorOffsets {
    /// Wraps the offsets of an [`AllocationsPml4tIterator`]
    Pml4t(PageTableEntryOffset),
    /// Wraps the offsets of an [`AllocationsPdptIterator`]
    Pdpt(PageTableEntryOffset,PageTableEntryOffset),
    /// Wraps the offsets of an [`AllocationsPdtIterator`]
    Pdt(PageTableEntryOffset,PageTableEntryOffset,PageTableEntryOffset),
    /// Wraps the offsets of an [`AllocationsPtIterator`]
    Pt(PageTableEntryOffset,PageTableEntryOffset,PageTableEntryOffset,PageTableEntryOffset),
}

impl Add<PageTableEntryOffset> for AllocationsIteratorOffsets {
    type Output = Self;

    /// Adds the gives offset to the current offsets and switches type accordingly.
    ///
    /// ## Panics
    ///
    /// Panics if no more offsets can be added (namely, when trying to add another
    /// offset to a PT iterator)
    fn add(self, rhs:PageTableEntryOffset) -> Self::Output {
        match self {
            Self::Pml4t(pml4t_ofs) => Self::Pdpt(pml4t_ofs,rhs),
            Self::Pdpt(pml4t_ofs,pdpt_ofs) => Self::Pdt(pml4t_ofs,pdpt_ofs,rhs),
            Self::Pdt(pml4t_ofs,pdpt_ofs,pdt_ofs) => Self::Pt(pml4t_ofs,pdpt_ofs,pdt_ofs,rhs),
            Self::Pt(..) => crate::panic("Adding offset to PT allocations iterator"),
        }
    }
}


/// Wrapper for other allocation iterators
pub(in crate::memory::paging)
enum AllocationsIterator {
    /// Wraps an [`AllocationsPml4tIterator`]
    Pml4t(AllocationsPml4tIterator),
    /// Wraps an [`AllocationsPdptIterator`]
    Pdpt(AllocationsPdptIterator),
    /// Wraps an [`AllocationsPdtIterator`]
    Pdt(AllocationsPdtIterator),
    /// Wraps an [`AllocationsPtIterator`]
    Pt(AllocationsPtIterator),
}

impl AllocationsIterator {
    /// Creates a new [`AllocationsIterator`] from the given [`AllocationsIteratorOffsets`]
    pub(in crate::memory::paging)
    fn from_offsets(offsets:AllocationsIteratorOffsets, owner:MemoryOwner) -> Option<Self> {
        match offsets {
            AllocationsIteratorOffsets::Pml4t(pml4t_ofs) => AllocationsPml4tIterator::new_with_offsets(pml4t_ofs, owner).map(|it| Self::from(it)),
            AllocationsIteratorOffsets::Pdpt(pml4t_ofs,pdpt_ofs) => AllocationsPdptIterator::new_with_offsets(pml4t_ofs,pdpt_ofs, owner).map(|it| Self::from(it)),
            AllocationsIteratorOffsets::Pdt(pml4t_ofs,pdpt_ofs,pdt_ofs) => AllocationsPdtIterator::new_with_offsets(pml4t_ofs,pdpt_ofs,pdt_ofs, owner).map(|it| Self::from(it)),
            AllocationsIteratorOffsets::Pt(pml4t_ofs,pdpt_ofs,pdt_ofs,pt_ofs) => AllocationsPtIterator::new_with_offsets(pml4t_ofs,pdpt_ofs,pdt_ofs,pt_ofs, owner).map(|it| Self::from(it)),
        }
    }

    /// Returns the [`AllocationsIteratorOffsets`] of the wrapped iterator
    pub(in crate::memory::paging)
    fn offsets(&self) -> AllocationsIteratorOffsets {
        match self {
            Self::Pml4t(it) => it.offsets().into(),
            Self::Pdpt(it) => it.offsets().into(),
            Self::Pdt(it) => it.offsets().into(),
            Self::Pt(it) => it.offsets().into(),
        }
    }

    /// Builds a [`LogicalAddress`] using the _current indexes_ of the parent tables
    /// and the _next index_ of the primary table of the wrapped iterator
    ///
    /// ## Warning
    ///
    /// The old bits of `laddr` are not cleaned before adding the new bits and thus
    /// the initial address passed to this function shall be a freshly created one
    /// and *never* an already existing one
    pub(in crate::memory::paging)
    fn build_laddr(&self, laddr:LogicalAddress) -> LogicalAddress {
        match self {
            Self::Pml4t(it) => it.build_laddr(laddr),
            Self::Pdpt(it) => it.build_laddr(laddr),
            Self::Pdt(it) => it.build_laddr(laddr),
            Self::Pt(it) => it.build_laddr(laddr),
        }
    }
}

impl From<AllocationsPml4tIterator> for AllocationsIterator {
    fn from(pml4t_iter:AllocationsPml4tIterator) -> Self {
        Self::Pml4t(pml4t_iter)
    }
}

impl From<AllocationsPdptIterator> for AllocationsIterator {
    fn from(pdpt_iter:AllocationsPdptIterator) -> Self {
        Self::Pdpt(pdpt_iter)
    }
}

impl From<AllocationsPdtIterator> for AllocationsIterator {
    fn from(pdt_iter:AllocationsPdtIterator) -> Self {
        Self::Pdt(pdt_iter)
    }
}

impl From<AllocationsPtIterator> for AllocationsIterator {
    fn from(pt_iter:AllocationsPtIterator) -> Self {
        Self::Pt(pt_iter)
    }
}

impl Iterator for AllocationsIterator {
    type Item = (bool,PageTableEntry);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Pml4t(it) => it.next(),
            Self::Pdpt(it) => it.next(),
            Self::Pdt(it) => it.next(),
            Self::Pt(it) => it.next(),
        }
    }
}


/// Iterates through all the entries of all the PT tables which are used to allocate memory
///
/// The iterator will visit all the entries of a PT. When the last entry is reached, it
/// will switch to the next entry of the parent PDT: if it points to another PT, that
/// table becomes the current table, otherwise the next entry of the PDT is checked. When
/// the last entry of the parent PDT is reached, the iterator will switch to the next entry
/// of the grand-parent PDPT, check all of its entries until one is found that points to
/// anothed PDT, which becomes the parent table, and so on and so forth. When the last entry
/// of the grand-parent is also reached, the iterator will switch to the next entry of the
/// grand-grand-parent PML4T and repeat again the same logic. The process is repeated until
/// all the available PTs used for memory allocation are visited.
pub(in crate::memory::paging)
struct AllocationsPtIterator {
    /// The parent tables iterator
    pdt_iter : AllocationsPdtIterator,
    /// The iterator over the current PT
    pt_iter : PageTableIterator,
}

impl AllocationsPtIterator {
    /// Creates a new [`AllocationsPtIterator`] based on the given [`MemoryOwner`]
    pub(in crate::memory::paging)
    fn new(
        owner:MemoryOwner
    ) -> Option<Self> {
        if let Some(mut pdt_iter) = AllocationsPdtIterator::new(owner) {
            if let Some((_,pdt_entry)) = pdt_iter.next() {
                if let Some(pt) = PageTable::try_from(pdt_entry).log_err().ok() {
                    let pt_iter = PageTableIterator::allocations(pt);
                    let iter = Self { pdt_iter, pt_iter };
                    return Some(iter);
                }
            }
        }
        None
    }

    /// Creates a new [`AllocationsPtIterator`] based on the given [`MemoryOwner`]
    /// and using the given offsets
    pub(in crate::memory::paging)
    fn new_with_offsets(
        pml4t_ofs:PageTableEntryOffset,
        pdpt_ofs:PageTableEntryOffset,
        pdt_ofs:PageTableEntryOffset,
        pt_ofs:PageTableEntryOffset,
        owner:MemoryOwner,
    ) -> Option<Self> {
        if let Some(mut pdt_iter) = AllocationsPdtIterator::new_with_offsets(pml4t_ofs, pdpt_ofs, pdt_ofs, owner) {
            if let Some((_,pdt_entry)) = pdt_iter.next() {
                if let Some(pt) = PageTable::try_from(pdt_entry).log_err().ok() {
                    let pt_iter = PageTableIterator::allocations_with_offset(pt, pt_ofs);
                    let iter = Self { pdt_iter, pt_iter };
                    return Some(iter);
                }
            }
        }
        None
    }

    /// Returns the _current offsets_ of the iterators
    pub(in crate::memory::paging)
    fn offsets(&self) -> AllocationsIteratorOffsets {
        self.pdt_iter.offsets() + self.pt_iter.offset()
    }

    /// Builds a [`LogicalAddress`] using the _current indexes_ of the iterators
    pub(in crate::memory::paging)
    fn build_laddr(&self, laddr:LogicalAddress) -> LogicalAddress {
        self.pdt_iter.build_laddr(laddr)
            .with_pt_index(self.pt_iter.table_index())
    }
}

impl Duplicate for AllocationsPtIterator {
    fn duplicate(&self) -> Self {
        Self {
            pdt_iter : self.pdt_iter.duplicate(),
            pt_iter  : self.pt_iter.duplicate(),
        }
    }

    fn duplicate_and_repeat(&self) -> Self {
        Self {
            pdt_iter : self.pdt_iter.duplicate(),
            pt_iter  : self.pt_iter.duplicate_and_repeat(),
        }
    }

    fn duplicate_and_advance(&self) -> Self {
        Self {
            pdt_iter : self.pdt_iter.duplicate(),
            pt_iter  : self.pt_iter.duplicate_and_advance(),
        }
    }
}

impl Iterator for AllocationsPtIterator {
    type Item = (bool,PageTableEntry);

    fn next(&mut self) -> Option<Self::Item> {
        let mut contiguous = true;
        if let Some(pt_entry) = self.pt_iter.next() {
            return Some((contiguous,pt_entry));
        }
        while let Some((cont,pdt_entry)) = self.pdt_iter.next() {
            let pdt_entry_bits = pdt_entry.bitmap();
            if !pdt_entry_bits.present() | pdt_entry_bits.page_size() {
                contiguous &= false;
                continue;
            }
            contiguous &= cont;
            self.pt_iter.reset(pdt_entry.into_or_panic());
            return self.pt_iter.next().map(|pt_entry| (contiguous, pt_entry));
        }
        None
    }
}


/// Iterates through all the entries of all the PDT tables which are used to allocate memory
///
/// The iterator will visit all the entries of a PDT. When the last entry is reached, it
/// will switch to the next entry of the parent PDPT: if it points to another PDT, that
/// will become the current table, otherwise the next entry of the PDPT is checked. When
/// the last entry of the parent PDPT is reached, the iterator will switch to the next entry
/// of the grand-parent PML4T, check all of its entries until one is found that points to
/// anothed PDPT, which becomes the parent table, and so on and so forth. The process is
/// repeated until all the available PDTs used for memory allocation are visited.
pub(in crate::memory::paging)
struct AllocationsPdtIterator {
    /// The parent tables iterator
    pdpt_iter : AllocationsPdptIterator,
    /// The iterator over the current PDT
    pdt_iter : PageTableIterator,
}

impl AllocationsPdtIterator {
    /// Creates a new [`AllocationsPdtIterator`] based on the given [`MemoryOwner`]
    pub(in crate::memory::paging)
    fn new(
        owner:MemoryOwner
    ) -> Option<Self> {
        if let Some(mut pdpt_iter) = AllocationsPdptIterator::new(owner) {
            if let Some((_,pdpt_entry)) = pdpt_iter.next() {
                if let Some(pdt) = PageTable::try_from(pdpt_entry).log_err().ok() {
                    let pdt_iter = PageTableIterator::allocations(pdt);
                    let iter = Self { pdpt_iter, pdt_iter };
                    return Some(iter);
                }
            }
        }
        None
    }

    /// Creates a new [`AllocationsPdtIterator`] based on the given [`MemoryOwner`]
    /// and using the given offsets
    pub(in crate::memory::paging)
    fn new_with_offsets(
        pml4t_ofs:PageTableEntryOffset,
        pdpt_ofs:PageTableEntryOffset,
        pdt_ofs:PageTableEntryOffset,
        owner:MemoryOwner,
    ) -> Option<Self> {
        if let Some(mut pdpt_iter) = AllocationsPdptIterator::new_with_offsets(pml4t_ofs, pdpt_ofs, owner) {
            if let Some((_,pdpt_entry)) = pdpt_iter.next() {
                if let Some(pdt) = PageTable::try_from(pdpt_entry).log_err().ok() {
                    let pdt_iter = PageTableIterator::allocations_with_offset(pdt, pdt_ofs);
                    let iter = Self { pdpt_iter, pdt_iter };
                    return Some(iter);
                }
            }
        }
        None
    }

    /// Returns the _current offsets_ of the iterators
    pub(in crate::memory::paging)
    fn offsets(&self) -> AllocationsIteratorOffsets {
        self.pdpt_iter.offsets() + self.pdt_iter.offset()
    }

    /// Builds a [`LogicalAddress`] using the _current indexes_ of the iterators
    pub(in crate::memory::paging)
    fn build_laddr(&self, laddr:LogicalAddress) -> LogicalAddress {
        self.pdpt_iter.build_laddr(laddr)
            .with_pdt_index(self.pdt_iter.table_index())
    }
}

impl Duplicate for AllocationsPdtIterator {
    fn duplicate(&self) -> Self {
        Self {
            pdpt_iter : self.pdpt_iter.duplicate(),
            pdt_iter  : self.pdt_iter.duplicate(),
        }
    }

    fn duplicate_and_repeat(&self) -> Self {
        Self {
            pdpt_iter : self.pdpt_iter.duplicate(),
            pdt_iter  : self.pdt_iter.duplicate_and_repeat(),
        }
    }

    fn duplicate_and_advance(&self) -> Self {
        Self {
            pdpt_iter : self.pdpt_iter.duplicate(),
            pdt_iter  : self.pdt_iter.duplicate_and_advance(),
        }
    }
}

impl Iterator for AllocationsPdtIterator {
    type Item = (bool,PageTableEntry);

    fn next(&mut self) -> Option<Self::Item> {
        let mut contiguous = true;
        if let Some(pdt_entry) = self.pdt_iter.next() {
            return Some((contiguous,pdt_entry));
        }
        while let Some((cont,pdpt_entry)) = self.pdpt_iter.next() {
            let pdpt_entry_bits = pdpt_entry.bitmap();
            if !pdpt_entry_bits.present() | pdpt_entry_bits.page_size() {
                contiguous &= false;
                continue;
            }
            contiguous &= cont;
            self.pdt_iter.reset(pdpt_entry.into_or_panic());
            return self.pdt_iter.next().map(|pdt_entry| (contiguous, pdt_entry));
        }
        None
    }
}


/// Iterates through all the entries of all the PDPT tables which are used to allocate memory
///
/// The iterator will visit all the entries of a PDPT. When the last entry is reached, it
/// will switch to the next entry of the parent PML4T: if it points to another PDPT, that
/// table will become the current table, otherwise the next entry of the PML4T is checked.
/// The process is repeated until all the available PDPTs used for memory allocation
/// are visited.
pub(in crate::memory::paging)
struct AllocationsPdptIterator {
    /// The parent tables iterator
    pml4t_iter : AllocationsPml4tIterator,
    /// The iterator over the current PDPT
    pdpt_iter : PageTableIterator,
}

impl AllocationsPdptIterator {
    /// Creates a new [`AllocationsPdptIterator`] based on the given [`MemoryOwner`]
    pub(in crate::memory::paging)
    fn new(
        owner:MemoryOwner
    ) -> Option<Self> {
        if let Some(mut pml4t_iter) = AllocationsPml4tIterator::new(owner) {
            if let Some((_,pml4t_entry)) = pml4t_iter.next() {
                if let Some(pdpt) = PageTable::try_from(pml4t_entry).log_err().ok() {
                    let pdpt_iter = PageTableIterator::allocations(pdpt);
                    let iter = Self { pml4t_iter, pdpt_iter };
                    return Some(iter);
                }
            }
        }
        None
    }

    /// Creates a new [`AllocationsPdptIterator`] based on the given [`MemoryOwner`]
    /// and using the given offsets
    pub(in crate::memory::paging)
    fn new_with_offsets(
        pml4t_ofs:PageTableEntryOffset,
        pdpt_ofs:PageTableEntryOffset,
        owner:MemoryOwner,
    ) -> Option<Self> {
        if let Some(mut pml4t_iter) = AllocationsPml4tIterator::new_with_offsets(pml4t_ofs, owner) {
            if let Some((_,pml4t_entry)) = pml4t_iter.next() {
                if let Some(pdpt) = PageTable::try_from(pml4t_entry).log_err().ok() {
                    let pdpt_iter = PageTableIterator::allocations_with_offset(pdpt, pdpt_ofs);
                    let iter = Self { pml4t_iter, pdpt_iter };
                    return Some(iter);
                }
            }
        }
        None
    }

    /// Returns the _current offsets_ of the iterators
    pub(in crate::memory::paging)
    fn offsets(&self) -> AllocationsIteratorOffsets {
        self.pml4t_iter.offsets() + self.pdpt_iter.offset()
    }

    /// Builds a [`LogicalAddress`] using the _current indexes_ of the iterators
    pub(in crate::memory::paging)
    fn build_laddr(&self, laddr:LogicalAddress) -> LogicalAddress {
        self.pml4t_iter.build_laddr(laddr)
            .with_pdpt_index(self.pdpt_iter.table_index())
    }
}

impl Duplicate for AllocationsPdptIterator {
    fn duplicate(&self) -> Self {
        Self {
            pml4t_iter : self.pml4t_iter.duplicate(),
            pdpt_iter  : self.pdpt_iter.duplicate(),
        }
    }

    fn duplicate_and_repeat(&self) -> Self {
        Self {
            pml4t_iter : self.pml4t_iter.duplicate(),
            pdpt_iter  : self.pdpt_iter.duplicate_and_repeat(),
        }
    }

    fn duplicate_and_advance(&self) -> Self {
        Self {
            pml4t_iter : self.pml4t_iter.duplicate(),
            pdpt_iter  : self.pdpt_iter.duplicate_and_advance(),
        }
    }
}

impl Iterator for AllocationsPdptIterator {
    type Item = (bool,PageTableEntry);

    fn next(&mut self) -> Option<Self::Item> {
        let mut contiguous = true;
        if let Some(pdpt_entry) = self.pdpt_iter.next() {
            return Some((contiguous,pdpt_entry));
        }
        while let Some((_,pml4t_entry)) = self.pml4t_iter.next() {
            let pml4t_entry_bits = pml4t_entry.bitmap();
            if !pml4t_entry_bits.present() {
                contiguous &= false;
                continue;
            }
            self.pdpt_iter.reset(pml4t_entry.into_or_panic());
            return self.pdpt_iter.next().map(|pdpt_entry| (contiguous, pdpt_entry));
        }
        None
    }
}


/// Iterates through all the entries of a PML4T table that are used to allocate memory
///
/// The iterator will visit all the entries of the PML4T that are used for memory allocation
pub(in crate::memory::paging)
struct AllocationsPml4tIterator {
    /// The iterator over the PML4T
    pml4t_iter : PageTableIterator,
}

impl AllocationsPml4tIterator {
    /// Creates a new [`AllocationsPml4tIterator`] based on the given [`MemoryOwner`]
    pub(in crate::memory::paging)
    fn new(
        owner:MemoryOwner
    ) -> Option<Self> {
        let pml4t = PageTable::pml4t(owner);
        let pml4t_iter = PageTableIterator::allocations(pml4t);
        let iter = Self { pml4t_iter };
        Some(iter)
    }

    /// Creates a new [`AllocationsPml4tIterator`] based on the given [`MemoryOwner`]
    /// and using the given offset
    pub(in crate::memory::paging)
    fn new_with_offsets(
        pml4t_ofs:PageTableEntryOffset,
        owner:MemoryOwner,
    ) -> Option<Self> {
        let pml4t = PageTable::pml4t(owner);
        let pml4t_iter = PageTableIterator::allocations_with_offset(pml4t, pml4t_ofs);
        let iter = Self { pml4t_iter };
        Some(iter)
    }

    /// Returns the _current offset_ of the iterator
    pub(in crate::memory::paging)
    fn offsets(&self) -> AllocationsIteratorOffsets {
        AllocationsIteratorOffsets::Pml4t(self.pml4t_iter.offset())
    }

    /// Builds a [`LogicalAddress`] using the _actual index_ of the iterator
    pub(in crate::memory::paging)
    fn build_laddr(&self, laddr:LogicalAddress) -> LogicalAddress {
        laddr.with_pml4t_index(self.pml4t_iter.table_index())
    }
}

impl Duplicate for AllocationsPml4tIterator {
    fn duplicate(&self) -> Self {
        Self {
            pml4t_iter : self.pml4t_iter.duplicate(),
        }
    }

    fn duplicate_and_repeat(&self) -> Self {
        Self {
            pml4t_iter : self.pml4t_iter.duplicate_and_repeat(),
        }
    }

    fn duplicate_and_advance(&self) -> Self {
        Self {
            pml4t_iter : self.pml4t_iter.duplicate_and_advance(),
        }
    }
}

impl Iterator for AllocationsPml4tIterator {
    type Item = (bool,PageTableEntry);

    fn next(&mut self) -> Option<Self::Item> {
        self.pml4t_iter.next()
            .map(|pml4t_entry| (true,pml4t_entry))
    }
}
