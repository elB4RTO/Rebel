use crate::memory::paging::*;
use crate::memory::paging::iterators::*;


/// Iterates through all the PDT entries of the tables used to trace allocations
///
/// This iterator stops at the first PDPT table entry not having the present
/// bit set, meaning that all PDT tables entries (which belong to an existing
/// PDPT table) are returned regardless of whether they point to an actual
/// tracing page or not.
pub(in crate::memory::paging)
struct TracingPDTEntriesIterator {
    pdpt_iter : PageTableIterator,
    pdt_iter  : PageTableIterator,
}

impl TracingPDTEntriesIterator {
    /// Creates a new [`TracingPDTEntriesIterator`] based on the given [`MemoryOwner`]
    pub(in crate::memory::paging)
    fn new(owner:MemoryOwner) -> Self {
        let pml4t = PageTable::pml4t(owner);
        let pml4t_ofs = PageTableEntryOffset::tracing(PageTableType::PageMapLevel4Table, owner).get_or_panic();
        let pdpt : PageTable = pml4t.at_offset_unchecked(&pml4t_ofs).into_or_panic();
        let pdpt_ofs = PageTableEntryOffset::tracing(PageTableType::PageDirectoryPointerTable, owner).get_or_panic();
        let pdt : PageTable = pdpt.at_offset_unchecked(&pdpt_ofs).into_or_panic();
        let pdt_ofs = PageTableEntryOffset::tracing(PageTableType::PageDirectoryTable, owner).get_or_panic();
        Self {
            pdpt_iter : PageTableIterator::tracing_with_offset(pdpt, pdpt_ofs),
            pdt_iter  : PageTableIterator::tracing_with_offset(pdt, pdt_ofs),
        }
    }
}

impl Iterator for TracingPDTEntriesIterator {
    type Item = PageTableEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(pdt_entry) = self.pdt_iter.next() {
            return Some(pdt_entry);
        }
        if let Some(pdpt_entry) = self.pdpt_iter.next() {
            if !pdpt_entry.bitmap().present() {
                return None;
            }
            self.pdt_iter.reset(pdpt_entry.into_or_panic());
            return self.next();
        }
        None
    }
}

/// Iterates through all the PDPT entries of the tables used to trace allocations
///
/// This iterator stops at the end of the PDPT table reserved for tracing, meaning
/// that all the PDPT table entries are returned regardless of whether they point
/// to an actual PDT table or not.
pub(in crate::memory::paging)
struct TracingPDPTEntriesIterator {
    pdpt_iter : PageTableIterator,
}

impl TracingPDPTEntriesIterator {
    /// Creates a new [`TracingPDPTEntriesIterator`] based on the given [`MemoryOwner`]
    pub(in crate::memory::paging)
    fn new(owner:MemoryOwner) -> Self {
        let pml4t = PageTable::pml4t(owner);
        let pml4t_ofs = PageTableEntryOffset::tracing(PageTableType::PageMapLevel4Table, owner).get_or_panic();
        let pdpt : PageTable = pml4t.at_offset_unchecked(&pml4t_ofs).into_or_panic();
        let pdpt_ofs = PageTableEntryOffset::tracing(PageTableType::PageDirectoryPointerTable, owner).get_or_panic();
        Self {
            pdpt_iter : PageTableIterator::tracing_with_offset(pdpt, pdpt_ofs),
        }
    }
}

impl Iterator for TracingPDPTEntriesIterator {
    type Item = PageTableEntry;

    fn next(&mut self) -> Option<Self::Item> {
        self.pdpt_iter.next()
    }
}


/// Iterates through all the entries of the tables used to trace allocations
///
/// This iterator stops at the first entry not having the present bit set,
/// regardless of whether it's a PDPT or PDT table entry. Namely, this iterator
/// only iterates existing tracing pages.
pub(in crate::memory::paging)
struct TracingPagesIterator {
    iter : TracingPDTEntriesIterator,
}

impl TracingPagesIterator {
    /// Creates a new [`TracingPagesIterator`] based on the given [`MemoryOwner`]
    pub(in crate::memory::paging)
    fn new(owner:MemoryOwner) -> Self {
        Self {
            iter : TracingPDTEntriesIterator::new(owner),
        }
    }
}

impl Iterator for TracingPagesIterator {
    type Item = PageTableEntry;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().and_then(|pdt_entry| match pdt_entry.bitmap().present() {
            true  => Some(pdt_entry),
            false => None,
        })
    }
}
