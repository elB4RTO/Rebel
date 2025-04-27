pub(in crate::memory::paging) mod clean;
pub(in crate::memory::paging) mod create;
pub(in crate::memory::paging) mod delete;
pub(in crate::memory::paging) mod drop;
pub(in crate::memory::paging) mod insert;
pub(in crate::memory::paging) mod merge;
pub(in crate::memory::paging) mod query;
pub(in crate::memory::paging) mod remove;
pub(in crate::memory::paging) mod search;
pub(in crate::memory::paging) mod take;
pub(in crate::memory::paging) mod update;


use crate::memory::MemoryOwner;
use crate::memory::paging::PageType;
use crate::memory::paging::bitmap::{Bitmap, PS_BIT};


pub(in crate::memory::paging::tracing) const
TRACE_PAGE_TYPE     : PageType = PageType::TwoMiB;

pub(in crate::memory::paging::tracing) const
TRACE_PAGE_SIZE     : u64 = TRACE_PAGE_TYPE.size();

pub(in crate::memory::paging::tracing) const
TRACE_PAGE_OWNER    : MemoryOwner = MemoryOwner::Kernel;

pub(in crate::memory::paging::tracing) const
TRACE_PAGE_FLAGS    : Bitmap = Bitmap::default_kernel().with_bits(PS_BIT);
