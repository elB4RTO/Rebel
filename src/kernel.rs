use crate::tty::*;

const KERNEL_NAME : &str = env!("CARGO_PKG_NAME");
const KERNEL_VERSION : &str = env!("CARGO_PKG_VERSION");

pub(crate)
fn start() {
    welcome();

    crate::idt::init_idt();
    crate::idt::enable_interrupts();

    crate::memory::init();
    crate::memory::init_kernel_tracing_pages();

    #[cfg(feature="unit_tests")]
    {
        crate::memory::tests::run();
        crate::tty::print("\n\n");
    }

    crate::memory::book_kernel_allocations_space();

    loop { crate::halt(); }
}


fn welcome() {
    clear();
    print(KERNEL_NAME);
    print(" v");
    print(KERNEL_VERSION);
    print("\n");
}
