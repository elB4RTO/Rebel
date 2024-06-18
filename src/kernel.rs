use crate::tty::*;

pub(crate) fn start() {
    clear();
    print("Welcome in the kernel");
    loop {}
}


#[panic_handler]
fn panic(msg:&core::panic::PanicInfo) -> ! {
    loop {}
}
