use crate::memory::paging::*;
use crate::memory::paging::iterators::*;


/// Iterates the entrie of a page table in the range defined its the indexer
pub(in crate::memory::paging)
struct PageTableIterator {
    /// The page table being iterated
    page_table : PageTable,
    /// The indexer to use for the iteration
    entry_offset : PageTableEntryOffset,
}

impl PageTableIterator {
    /// Creates a new [`PageTableIterator`] for the given [`PageTable`] and
    /// using the given [`PageTableEntryOffset`] as indexer
    const
    fn new(page_table:PageTable, entry_offset:PageTableEntryOffset) -> Self {
        Self {
            page_table, entry_offset
        }
    }

    /// Creates a new [`PageTableIterator`] upon the given table, bounded to
    /// the allocation tables only
    pub(in crate::memory::paging) const
    fn allocations(page_table:PageTable) -> Self {
        let entry_offset = PageTableEntryOffset::allocations(page_table.table_type, page_table.table_owner);
        Self::new(page_table, entry_offset)
    }

    /// Creates a new [`PageTableIterator`] upon the given table and with the
    /// given offset, bounded to the allocation tables only
    pub(in crate::memory::paging) const
    fn allocations_with_offset(page_table:PageTable, entry_offset:PageTableEntryOffset) -> Self {
        Self::new(page_table, entry_offset)
    }

    /// Creates a new [`PageTableIterator`] upon the given table and with the
    /// given index, bounded to the allocation tables only
    pub(in crate::memory::paging)
    fn allocations_with_index(page_table:PageTable, entry_index:PageTableEntryIndex) -> Self {
        Self::new(page_table, entry_index.into())
    }

    /// Creates a new [`PageTableIterator`] upon the given table, bounded to
    /// the tracing tables only
    pub(in crate::memory::paging)
    fn tracing(page_table:PageTable) -> Result<Self,PagingError> {
        let entry_offset = PageTableEntryOffset::tracing(page_table.table_type, page_table.table_owner)?;
        Ok(Self::new(page_table, entry_offset))
    }

    /// Creates a new [`PageTableIterator`] upon the given table and with the
    /// given offset, bounded to the tracing tables only
    pub(in crate::memory::paging) const
    fn tracing_with_offset(page_table:PageTable, entry_offset:PageTableEntryOffset) -> Self {
        Self::new(page_table, entry_offset)
    }

    /// Reverses the iteration order
    pub(in crate::memory::paging)
    fn reversed(mut self) -> Self {
        self.entry_offset.reverse();
        self
    }

    /// Resets the [`PageTableIterator`] by replacing the current [`PageTable`] with
    /// the given [`PageTable`] and restarting the indexer
    ///
    /// ## Warning
    ///
    /// Care must be taken to ensure that `new_table` is a [`PageTable`] of the same
    /// type of the original table used to create the [`PageTableIterator`]. Using
    /// a table of a different type may cause the [`PageTableIterator`] to return
    /// un-expected table entries.
    pub(in crate::memory::paging)
    fn reset(&mut self, new_table:PageTable) {
        self.page_table = new_table;
        self.entry_offset.restart();
    }

    /// Sets the [`PageTableIterator`] to a state such that it cannot iterate further
    pub(in crate::memory::paging)
    fn terminate(&mut self) {
        self.entry_offset.terminate()
    }

    /// Returns the [`PageTable`] pointed to by the table entry at the current
    /// index
    ///
    /// ## Warning
    ///
    /// This function does not check whether the stored table can actually have
    /// child table, not whether the index refers to an entry that actually
    /// points to an existing table, nor whether the index is inside the bounds
    /// defined by the range or by the table itself.
    pub(in crate::memory::paging)
    fn get_unchecked(&self) -> PageTable {
        self.page_table.at_offset_unchecked(&self.entry_offset).into_or_panic()
    }

    /// Checks whether the [`PageTableIterator`] cannot iterate further
    ///
    /// ## Returns
    ///
    /// Returns `true` if the iterator is terminated, `false` otherwise
    pub(in crate::memory::paging)
    fn cannot_iterate_further(&self) -> bool {
        !self.entry_offset.has_next()
    }

    /// Returns the _current index_
    pub(in crate::memory::paging)
    fn table_index(&self) -> u64 {
        self.entry_offset.index()
    }

    /// Returns a copy of the iterator with the _current offset_ repeated
    pub(in crate::memory::paging)
    fn offset(&self) -> PageTableEntryOffset {
        self.entry_offset.duplicate_and_repeat()
    }
}

impl Duplicate for PageTableIterator {
    fn duplicate(&self) -> Self {
        Self {
            page_table   : self.page_table.clone(),
            entry_offset : self.entry_offset.duplicate(),
        }
    }

    fn duplicate_and_repeat(&self) -> Self {
        Self {
            page_table   : self.page_table.clone(),
            entry_offset : self.entry_offset.duplicate_and_repeat(),
        }
    }

    fn duplicate_and_advance(&self) -> Self {
        Self {
            page_table   : self.page_table.clone(),
            entry_offset : self.entry_offset.duplicate_and_advance(),
        }
    }
}

impl Iterator for PageTableIterator {
    type Item = PageTableEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(offset) = self.entry_offset.next() {
            let entry_paddr = self.page_table.base_address + offset;
            return Some(PageTableEntry::new(entry_paddr, self.page_table.table_type, self.page_table.table_owner));
        }
        None
    }
}


/// Same as [`PageTableIterator`], but borrows the table instead of copying it
pub(in crate::memory::paging)
struct PageTableRefIterator<'l> {
    /// The page table being iterated
    page_table : &'l PageTable,
    /// The indexer to use for the iteration
    entry_offset : PageTableEntryOffset,
}

impl<'l> PageTableRefIterator<'l> {
    /// Creates a new [`PageTableRefIterator`] upon the given [`PageTable`], bounded
    /// to the allocation tables only
    pub(in crate::memory::paging) const
    fn allocations(page_table:&'l PageTable) -> Self {
        let entry_offset = PageTableEntryOffset::allocations(page_table.table_type, page_table.table_owner);
        Self {
            page_table, entry_offset
        }
    }

    /// Creates a new [`PageTableRefIterator`] upon the given [`PageTable`] and with the
    /// given index, bounded to the allocation tables only
    pub(in crate::memory::paging) const
    fn allocations_with_offset(page_table:&'l PageTable, entry_offset:PageTableEntryOffset) -> Self {
        Self {
            page_table, entry_offset
        }
    }
}

impl<'l> Iterator for PageTableRefIterator<'l> {
    type Item = PageTableEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(offset) = self.entry_offset.next() {
            let entry_paddr = self.page_table.base_address + offset;
            return Some(PageTableEntry::new(entry_paddr, self.page_table.table_type, self.page_table.table_owner));
        }
        None
    }
}


/// Same as [`PageTableIterator`], but uses absolute bounds
pub(in crate::memory::paging)
struct UnboundedPageTableIterator {
    /// The page table being iterated
    page_table : PageTable,
    /// The indexer to use for the iteration
    entry_offset : PageTableEntryOffset,
}

impl UnboundedPageTableIterator {
    /// Creates a new [`UnboundedPageTableIterator`] upon the given [`PageTable`],
    /// without any bounds
    pub(in crate::memory::paging) const
    fn new(table:PageTable) -> Self {
        Self {
            page_table   : table,
            entry_offset : PageTableEntryOffset::unbounded(),
        }
    }
}

impl Iterator for UnboundedPageTableIterator {
    type Item = PageTableEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(offset) = self.entry_offset.next() {
            let entry_paddr = self.page_table.base_address + offset;
            return Some(PageTableEntry::new(entry_paddr, self.page_table.table_type, self.page_table.table_owner));
        }
        None
    }
}


/// Enumerates all the entries of a page table in the range defined its the indexer
pub(in crate::memory)
struct PageTableEnumerator {
    /// The iterator constructed upon the page table to iterate
    table_iter : PageTableIterator,
    /// The indexer used to provide enumeration indexes
    entry_index : PageTableEntryIndex,
}

impl PageTableEnumerator {
    /// Creates a new [`PageTableEnumerator`] upon the given [`PageTable`], bounded
    /// to the allocation tables only
    pub(in crate::memory::paging) const
    fn allocations(page_table:PageTable) -> Self {
        let entry_index = PageTableEntryIndex::allocations(page_table.table_type, page_table.table_owner);
        let table_iter = PageTableIterator::allocations(page_table);
        Self {
            table_iter, entry_index
        }
    }
}

impl From<PageTableIterator> for PageTableEnumerator {
    fn from(table_iter:PageTableIterator) -> Self {
        let entry_index = table_iter.entry_offset.duplicate().into();
        Self {
            table_iter, entry_index,
        }
    }
}

impl Iterator for PageTableEnumerator {
    type Item = (u64,PageTableEntry);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(entry) = self.table_iter.next() {
            return Some((self.entry_index.next_unchecked(), entry));
        }
        None
    }
}


/// Same as [`PageTableEnumerator`], but borrows the table instead of copying it
pub(in crate::memory::paging)
struct PageTableRefEnumerator<'l> {
    /// The ref iterator constructed upon the page table to iterate
    table_iter : PageTableRefIterator<'l>,
    /// The indexer used to provide enumeration indexes
    entry_index : PageTableEntryIndex,
}

impl<'l> PageTableRefEnumerator<'l> {
    /// Creates a new [`PageTableRefEnumerator`] upon the given table, bounded
    /// to the allocation tables only
    pub(in crate::memory::paging) const
    fn allocations(page_table:&'l PageTable) -> Self {
        let entry_index = PageTableEntryIndex::allocations(page_table.table_type, page_table.table_owner);
        let table_iter = PageTableRefIterator::allocations(page_table);
        Self {
            table_iter, entry_index
        }
    }
}

impl<'l> From<PageTableRefIterator<'l>> for PageTableRefEnumerator<'l> {
    fn from(table_iter:PageTableRefIterator<'l>) -> Self {
        let entry_index = table_iter.entry_offset.duplicate().into();
        Self {
            table_iter, entry_index,
        }
    }
}

impl<'l> Iterator for PageTableRefEnumerator<'l> {
    type Item = (u64,PageTableEntry);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(entry) = self.table_iter.next() {
            return Some((self.entry_index.next_unchecked(), entry));
        }
        None
    }
}


/// Same as [`PageTableEnumerator`], but uses absolute bounds
pub(in crate::memory)
struct UnboundedPageTableEnumerator {
    /// The iterator constructed upon the page table to iterate
    table_iter : UnboundedPageTableIterator,
    /// The indexer used to provide enumeration indexes
    entry_index : PageTableEntryIndex,
}

impl UnboundedPageTableEnumerator {
    /// Creates a new [`UnboundedPageTableEnumerator`] upon the given table,
    /// without any bounds
    pub(in crate::memory::paging) const
    fn new(table:PageTable) -> Self {
        Self {
            table_iter  : UnboundedPageTableIterator::new(table),
            entry_index : PageTableEntryIndex::unbounded(),
        }
    }
}

impl From<UnboundedPageTableIterator> for UnboundedPageTableEnumerator {
    fn from(table_iter:UnboundedPageTableIterator) -> Self {
        let entry_index = table_iter.entry_offset.duplicate().into();
        Self {
            table_iter, entry_index
        }
    }
}

impl Iterator for UnboundedPageTableEnumerator {
    type Item = (u64,PageTableEntry);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(entry) = self.table_iter.next() {
            return Some((self.entry_index.next_unchecked(), entry));
        }
        None
    }
}
