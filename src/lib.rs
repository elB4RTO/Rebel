#![no_std]
#![no_main]

mod kernel;

pub(crate) mod cpu;
pub(crate) mod memory;
pub(crate) mod panic;
pub(crate) mod traits;
pub(crate) mod tty;

#[cfg(feature="unit_tests")]
pub(crate) mod test;

pub(crate) use crate::panic::*;
pub(crate) use crate::traits::*;


#[no_mangle]
pub extern "C"
fn kernel_main() {
    kernel::start();
}


pub(crate)
fn halt() {
    unsafe {
        core::arch::asm!("hlt");
    }
}
