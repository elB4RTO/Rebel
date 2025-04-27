use crate::halt;
use crate::tty::*;

use core::panic::PanicInfo;


pub(crate)
trait Panic {
    fn panic(&self) -> !;
}


#[panic_handler]
fn rust_panic(_:&PanicInfo) -> ! {
    print("\n!!! CORE PANIC !!!\n");
    loop {
        halt();
    }
}


pub(crate)
fn panic(msg:&str) -> ! {
    print("\n!!! KERNEL PANIC !!!\n");
    print(msg);
    loop {
        halt();
    }
}
