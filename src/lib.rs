#![no_std]
#![no_main]

mod kernel;

pub(crate) mod tty;

#[no_mangle]
pub extern "C" fn kernel_main() {
    kernel::start();
}
