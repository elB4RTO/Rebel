pub(in crate::memory::paging) mod allocations;
pub(in crate::memory::paging) mod indexers;
pub(in crate::memory::paging) mod tables;
pub(in crate::memory::paging) mod tracing;

pub(in crate::memory::paging) use allocations::*;
pub(in crate::memory) use indexers::*;
pub(in crate::memory::paging) use tables::*;


use crate::memory::paging::*;

use core::ops::Range;


pub(in crate::memory::paging) const
KERNEL_ALLOCATIONS_PML4T_INDEX_RANGE    : Range<u64> = KERNEL_ALLOCATIONS_PML4T_FIRST_INDEX..KERNEL_ALLOCATIONS_PML4T_LIMIT_INDEX;

pub(in crate::memory::paging) const
USER_ALLOCATIONS_PML4T_INDEX_RANGE      : Range<u64> = USER_ALLOCATIONS_PML4T_FIRST_INDEX..USER_ALLOCATIONS_PML4T_LIMIT_INDEX;


pub(in crate::memory::paging) const
KERNEL_ALLOCATIONS_PML4T_OFFSET_RANGE   : Range<u64> = KERNEL_ALLOCATIONS_PML4T_FIRST_OFFSET..KERNEL_ALLOCATIONS_PML4T_LIMIT_OFFSET;

pub(in crate::memory::paging) const
USER_ALLOCATIONS_PML4T_OFFSET_RANGE     : Range<u64> = USER_ALLOCATIONS_PML4T_FIRST_OFFSET..USER_ALLOCATIONS_PML4T_LIMIT_OFFSET;


pub(in crate::memory::paging) const
TRACING_TABLES_PML4T_OFFSET_RANGE       : Range<u64> = TRACING_TABLES_PML4T_OFFSET..LIMIT_ENTRY_OFFSET;

pub(in crate::memory::paging) const
KERNEL_TRACING_TABLES_PDPT_OFFSET_RANGE : Range<u64> = KERNEL_TRACING_TABLES_PDPT_FIRST_OFFSET..KERNEL_TRACING_TABLES_PDPT_LIMIT_OFFSET;

pub(in crate::memory::paging) const
USER_TRACING_TABLES_PDPT_OFFSET_RANGE   : Range<u64> = USER_TRACING_TABLES_PDPT_FIRST_OFFSET..USER_TRACING_TABLES_PDPT_LIMIT_OFFSET;


pub(in crate::memory)
trait Duplicate {
    /// Duplicate as is
    fn duplicate(&self) -> Self;

    /// Duplicate in such a way that the next call to [`Iterator::next()`]
    /// will return the current result again
    fn duplicate_and_repeat(&self) -> Self;

    /// Duplicate in such a way that the next call to [`Iterator::next()`]
    /// will return the next result
    fn duplicate_and_advance(&self) -> Self;
}
