use crate::memory::*;

use core::ops::{BitOr, BitAnd};


// Present
pub(in crate::memory) const
P_BIT   : u64 = 0b0000000000000000000000000000000000000000000000000000000000000001;

// Read/Write
pub(in crate::memory) const
RW_BIT  : u64 = 0b0000000000000000000000000000000000000000000000000000000000000010;

// Supervisor/User
pub(in crate::memory) const
SU_BIT  : u64 = 0b0000000000000000000000000000000000000000000000000000000000000100;

// Write Through
pub(in crate::memory) const
PWT_BIT : u64 = 0b0000000000000000000000000000000000000000000000000000000000001000;

// Cache Disable
pub(in crate::memory) const
PCD_BIT : u64 = 0b0000000000000000000000000000000000000000000000000000000000010000;

// Accessed
pub(in crate::memory) const
A_BIT   : u64 = 0b0000000000000000000000000000000000000000000000000000000000100000;

// Dirty
pub(in crate::memory) const
D_BIT   : u64 = 0b0000000000000000000000000000000000000000000000000000000001000000;

// Page Size
pub(in crate::memory) const
PS_BIT  : u64 = 0b0000000000000000000000000000000000000000000000000000000010000000;

// Global
pub(in crate::memory) const
G_BIT   : u64 = 0b0000000000000000000000000000000000000000000000000000000100000000;

// Page Attribute Table
pub(in crate::memory) const
PAT_BIT : u64 = 0b0000000000000000000000000000000000000000000000000001000000000000;

// Protection Key
pub(in crate::memory) const
PK_BIT  : u64 = 0b0111100000000000000000000000000000000000000000000000000000000000;

// Execute Disable
pub(in crate::memory) const
XD_BIT  : u64 = 0b1000000000000000000000000000000000000000000000000000000000000000;


// Bitmasks for the address bits of a logical address (defined at runtime)
pub(in crate::memory) static mut
BITMASK_ADDRESS_1G  : u64 = 0b0000000000000000000000000000000001000000000000000000000000000000;

pub(in crate::memory) static mut
BITMASK_ADDRESS_2M  : u64 = 0b0000000000000000000000000000000000000000001000000000000000000000;

pub(in crate::memory) static mut
BITMASK_ADDRESS_4K  : u64 = 0b0000000000000000000000000000000000000000000000000001000000000000;


// Bitmasks for the offset bits of a logical address
pub(in crate::memory) const
BITMASK_OFFSET_1G   : u64 = 0b0000000000000000000000000000000000111111111111111111111111111111;

pub(in crate::memory) const
BITMASK_OFFSET_2M   : u64 = 0b0000000000000000000000000000000000000000000111111111111111111111;

pub(in crate::memory) const
BITMASK_OFFSET_4K   : u64 = 0b0000000000000000000000000000000000000000000000000000111111111111;


/// Represents the bits of a table entry
#[derive(Clone,Copy)]
pub(in crate::memory)
struct Bitmap {
    bits : u64,
}

impl Bitmap {
    /// Returns a bitmap with all zeros
    pub(in crate::memory) const
    fn new() -> Self {
        Self::from_bits(0)
    }

    /// Returns a bitmap with the *Read/Write* bit and the *Present* bit
    pub(in crate::memory) const
    fn default_kernel() -> Self {
        Self::from_bits(RW_BIT|P_BIT)
    }

    /// Returns a bitmap with the *User/Supervisor* bit, the *Read/Write* bit
    /// and the *Present* bit
    pub(in crate::memory) const
    fn default_user() -> Self {
        Self::from_bits(SU_BIT|RW_BIT|P_BIT)
    }

    /// Returns the bits of the bitmap
    pub(in crate::memory) const
    fn bits(&self) -> u64 {
        self.bits
    }

    /// Creates a bitmap from the given bits
    pub(in crate::memory) const
    fn from_bits(bits:u64) -> Self {
        Self { bits }
    }

    /// Applies the given bits to the bitmap with and returns it
    pub(in crate::memory) const
    fn with_bits(mut self, bits:u64) -> Self {
        self.bits |= bits;
        self
    }

    /// Removes the given bits to the bitmap and returns it
    pub(in crate::memory) const
    fn without_bits(mut self, bits:u64) -> Self {
        self.bits &= !bits;
        self
    }

    /// Returns `true` if all the given bits are set, `false` otherwise
    pub(in crate::memory) const
    fn has_bits(&self, bits:u64) -> bool {
        (self.bits & bits) == bits
    }

    /// Returns the bits of the address for the given page type
    pub(in crate::memory)
    fn address(&self, page_type:PageType) -> u64 {
        unsafe {
            match page_type {
                PageType::OneGiB  => self.bits & BITMASK_ADDRESS_1G,
                PageType::TwoMiB  => self.bits & BITMASK_ADDRESS_2M,
                PageType::FourKiB => self.bits & BITMASK_ADDRESS_4K,
            }
        }
    }

    /// Whether the *Present* bit is set or not
    ///
    /// ## Description
    ///
    /// If this bit is set then the entry actually points to something in physical
    /// memory (may it be a page or another table).
    /// If an entry is accessed by the processor but this bit is not set, then a
    /// page fault occurs.
    pub(in crate::memory) const
    fn present(&self) -> bool {
        self.has_bits(P_BIT)
    }

    /// Whether the *Read/Write* bit is present or not
    ///
    /// ## Description
    ///
    /// If this bit is set, the entry is both readable and writable, otherwise
    /// the entry is read-only.
    /// In `CR0` this bit determines if this behavior only applies to userland (meaning
    /// that the kernel will have unrestricted write access, which is the default) or
    /// if it applies to both userland and kernelspace.
    pub(in crate::memory) const
    fn writable(&self) -> bool {
        self.has_bits(RW_BIT)
    }

    /// Whether the *User/Supervisor* bit is present or not
    ///
    /// ## Description
    ///
    /// If this bit is set then the entry may be accessed in user-mode, otherwise
    /// the entry can only be accessed in supervisor-mode.
    /// If the entry points to a table, this bit controls access to all the entries
    /// referenced within that table.
    pub(in crate::memory) const
    fn supervised(&self) -> bool {
        self.has_bits(SU_BIT)
    }

    /// Whether the *Write Through* bit is present or not
    ///
    /// ## Description
    ///
    /// If this bit is set then write-through caching is enabled, otherwise
    /// write-back is enabled instead.
    pub(in crate::memory) const
    fn write_through(&self) -> bool {
        self.has_bits(PWT_BIT)
    }

    /// Whether the *Cache Disable* bit is present or not
    ///
    /// ## Description
    ///
    /// If this bit is set then the entry won't be cached, otherwise it will be.
    pub(in crate::memory) const
    fn cache_disable(&self) -> bool {
        self.has_bits(PCD_BIT)
    }

    /// Whether the *Accessed* bit is present or not
    ///
    /// ## Description
    ///
    /// This bit is set by the processor and can be used to determine whether the entry
    /// has been accessed during logical address translation.
    /// This bit won't be cleared by the processor, so that burden falls to the system.
    pub(in crate::memory) const
    fn accessed(&self) -> bool {
        self.has_bits(A_BIT)
    }

    /// Whether the *Dirty* bit is present or not
    ///
    /// ## Description
    ///
    /// This bit is only available for the entries pointing to a pgage.
    /// This bit is set by the processor and can be used to determine whether the page
    /// has been written to.
    pub(in crate::memory) const
    fn dirty(&self) -> bool {
        self.has_bits(D_BIT)
    }

    /// Whether the *Page Size* bit is present or not
    ///
    /// ## Description
    ///
    /// This bit is only available for PDPT and PDT tables.
    /// If this bìt is set, the entry is mapped to point to a huge page.
    /// If the table is a PDPT then the page will be a 1 GiB page, if the
    /// table is a PDT then the page will be a 2 MiB page.
    pub(in crate::memory) const
    fn page_size(&self) -> bool {
        self.has_bits(PS_BIT)
    }

    /// Whether the *Global* bit is present or not
    ///
    /// ## Description
    ///
    /// This bit is only available for the entries pointing to a page.
    /// If this bit is set then the corresponding entry won't be invalidated
    /// by the processor when a MOV to CR3 instruction happens.
    /// The PGE bit (7th) must be set in CR4 in order to enable global pages.
    pub(in crate::memory) const
    fn global(&self) -> bool {
        self.has_bits(G_BIT)
    }

    /// Whether the *Page Attribute Table* bit is present or not
    ///
    /// ## Description
    ///
    /// This bit is only available for entries pointing to a page.
    /// If the table is a PDPT or a PDT then the bit is 12th position, if the table
    /// is a PD then the bit is in 7th position (since the Page Size bit cannot be set).
    pub(in crate::memory) const
    fn page_attribute_table(&self) -> bool {
        match self.page_size() {
            true  => self.has_bits(PAT_BIT),
            false => self.has_bits(PS_BIT),
        }
    }

    /// Returns the *Protection Key* bits
    ///
    /// ## Description
    ///
    /// The protection key is composed of 4 bits corresponding to each virtual address
    /// that is used to control user-mode and supervisor-mode memory access.
    /// If the PKE bit (22nd) in CR4 is set, then the PKRU register is used for determining
    /// access rights for user-mode based on the protection key.
    /// If the PKS bit (24th) is set in CR4, then the PKRS register is used for determining
    /// accesss rights in supervisor-mode based on the protection key.
    /// A protection key allows the system to enable/disable access rights for multiple page
    /// entries across different address spaces at once.
    pub(in crate::memory) const
    fn protection_key(&self) -> u64 {
        self.bits & PK_BIT
    }

    /// Whether the *Execute Disable* bit is present or not
    ///
    /// ## Description
    ///
    /// If the NXE bit (11th) is set in the EFER register, then instructions are not
    /// allowed to be executed at addresses within the entry whenever this bit is set.
    /// If the NXE bit is not set in the EFER register, this bit is reserved and
    /// should be set to 0.
    pub(in crate::memory) const
    fn esecute_disable(&self) -> bool {
        self.has_bits(XD_BIT)
    }
}

impl From<MemoryOwner> for Bitmap {
    fn from(owner: MemoryOwner) -> Self {
        match owner {
            MemoryOwner::Kernel => Self::default_kernel(),
            MemoryOwner::User => Self::default_user(),
        }
    }
}

impl From<PageType> for Bitmap {
    fn from(page_type:PageType) -> Self {
        match page_type {
            PageType::OneGiB|PageType::TwoMiB => Self::from_bits(PS_BIT|RW_BIT|P_BIT),
            PageType::FourKiB => Self::from_bits(RW_BIT|P_BIT),
        }
    }
}

impl From<PhysicalAddress> for Bitmap {
    fn from(addr:PhysicalAddress) -> Self {
        Self::from(addr.get())
    }
}

impl From<u64> for Bitmap {
    fn from(bits:u64) -> Self {
        Self {
            bits,
        }
    }
}

impl BitOr for Bitmap {
    type Output = Self;

    fn bitor(self, rhs:Self) -> Self::Output {
        Self {
            bits : self.bits | rhs.bits,
        }
    }
}

impl BitAnd for Bitmap {
    type Output = Self;

    fn bitand(self, rhs:Self) -> Self::Output {
        Self {
            bits : self.bits & rhs.bits,
        }
    }
}
