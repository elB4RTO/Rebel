use crate::memory::{
    SIZE_4KiB, SIZE_2MiB, SIZE_1GiB,
};
use crate::memory::address::*;
use crate::memory::map;
use crate::memory::paging::*;


/// Represents the page size of a page
#[derive(Clone,Copy,PartialEq)]
pub(in crate::memory)
enum PageType {
    OneGiB,
    TwoMiB,
    FourKiB,
}

impl PageType {
    pub(in crate::memory::paging) const
    fn size(&self) -> u64 {
        match self {
            PageType::OneGiB  => SIZE_1GiB,
            PageType::TwoMiB  => SIZE_2MiB,
            PageType::FourKiB => SIZE_4KiB,
        }
    }
}

impl TryFrom<PageTableType> for PageType {
    type Error = PagingError;

    /// Returns the page type that can be directly indexed in the given table type
    ///
    /// ## Returns
    ///
    /// Returns an [`Err`] if the table cannot directly store pages, otherwise
    /// returns an [`Ok`] with the [`PageType`].
    fn try_from(table_type:PageTableType) -> Result<Self, Self::Error> {
        use PageTableType::*;
        use PagingError::*;
        match table_type {
            PageMapLevel4Table => Err(IncompatiblePage),
            PageDirectoryPointerTable => Ok(Self::OneGiB),
            PageDirectoryTable => Ok(Self::TwoMiB),
            PageTable => Ok(Self::FourKiB),
        }
    }
}

impl Into<u64> for PageType {
    /// Returns the size in Bytes of the [`PageType`]
    fn into(self) -> u64 {
        self.size()
    }
}


/// Represents a page
pub(in crate::memory)
struct Page {
    /// The address of the page
    page_addr : TotalAddress,
    /// Determines the size of the page
    page_type : PageType,
    /// The owner of the page
    page_owner : MemoryOwner,
    /// The available memory left in the page
    free_space : u64,
}

impl TryFrom<PageTableEntry> for Page {
    type Error = PagingError;

    /// Returns a new page which type is the one that can be directly indexed
    /// in the table to which the given table entry belongs to
    ///
    /// ## Returns
    ///
    /// Returns an [`Err`] if the table cannot directly store pages, otherwise
    /// returns an [`Ok`] with the resulting [`Page`].
    fn try_from(entry:PageTableEntry) -> Result<Self, Self::Error> {
        let entry_bits = entry.bitmap();
        if !entry_bits.present() {
            return Err(PagingError::PageNotPresent);
        }
        let page_type = PageType::try_from(entry.table_type)?;
        if page_type != PageType::FourKiB && !entry_bits.page_size() {
            return Err(PagingError::PageNotHuge);
        }
        let page_owner = entry.table_owner;
        let page_paddr = PhysicalAddress::from(entry_bits.address(page_type));
        let page_laddr = match page_paddr.to_logical(page_owner) {
            Err(e) => return Err(PagingError::AddressError(e)),
            Ok(a) => a,
        };
        let page_addr = TotalAddress::new(page_paddr, page_laddr);
        let free_space = match map::get_space(page_paddr, page_type, page_owner) {
            Err(_) => return Err(PagingError::InvalidRequest),
            Ok(s) => s,
        };
        Ok(Self { page_addr, page_type, page_owner, free_space })
    }
}
