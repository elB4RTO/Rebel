#![no_std]
#![no_main]

pub(crate) mod kernel;

#[no_mangle]
pub extern "C" fn kernel_main() {
    kernel::start();
}
