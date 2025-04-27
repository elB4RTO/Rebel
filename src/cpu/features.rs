use core::arch::global_asm;


global_asm!(include_str!("cpuid.asm"));

extern "C" {
fn cpuid_address_size() -> u32;
}


/// Returns the physical and logical address size, respectively
pub(crate) fn pae_address_width() -> (u32, u32) {
    let value = unsafe { cpuid_address_size() };
    let phys_size = value & 0xFF;
    let virt_size = (value >> 8) & 0xFF;
    (phys_size, virt_size)
}
