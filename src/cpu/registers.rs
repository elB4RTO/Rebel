use core::arch::global_asm;


const
MSR_EFER_IA32       : u64 = 0xC0000080;

const
MSR_FS_BASE         : u64 = 0xC0000100;

const
MSR_GS_BASE         : u64 = 0xC0000101;

const
MSR_KERNEL_GS_BASE  : u64 = 0xC0000102;


global_asm!(include_str!("registers.asm"));

unsafe extern "C" {
    /// Reads the content of `CR0`
    pub(crate)
    fn get_cr0() -> u64;

    /// Writes the given value into `CR0`
    pub(crate)
    fn set_cr0(value:u64);

    /// Reads the content of `CR2`
    pub(crate)
    fn get_cr2() -> u64;

    /// Writes the given value into `CR2`
    pub(crate)
    fn set_cr2(value:u64);

    /// Reads the content of `CR3`
    pub(crate)
    fn get_cr3() -> u64;

    /// Writes the given value into `CR3`
    pub(crate)
    fn set_cr3(value:u64);

    /// Reads the content of `CR4`
    pub(crate)
    fn get_cr4() -> u64;

    /// Writes the given value into `CR4`
    pub(crate)
    fn set_cr4(value:u64);

    /// Reads the content of `CR8`
    pub(crate)
    fn get_cr8() -> u64;

    /// Writes the given value into `CR8`
    pub(crate)
    fn set_cr8(value:u64);

    /// Reads the content of `MSR` for the given address
    pub(crate)
    fn get_msr(address:u64) -> u64;

    /// Writes the given value into `MSR` for the given `MSR` address
    pub(crate)
    fn set_msr(address:u64, value:u64);
}

/// Reads the content of `MSR` using the address of `EFER`
pub(crate) unsafe
fn get_msr_efer() -> u64 {
    get_msr(MSR_EFER_IA32)
}

/// Writes the given value into `MSR` using the address of `EFER`
pub(crate) unsafe
fn set_msr_efer(value:u64) {
    set_msr(MSR_EFER_IA32, value)
}
