
use crate::memory::address::*;


/// Store both the physical and logical versions of an address
#[repr(align(8))]
#[derive(Clone,Copy)]
pub(in crate::memory) struct TotalAddress {
    pub physical : PhysicalAddress,
    pub logical  : LogicalAddress,
}

impl TotalAddress {
    pub(in crate::memory)
    fn new(physical:PhysicalAddress, logical:LogicalAddress) -> Self {
        Self { physical, logical }
    }
}
