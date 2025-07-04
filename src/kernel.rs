const KERNEL_NAME : &str = env!("CARGO_PKG_NAME");
const KERNEL_VERSION : &str = env!("CARGO_PKG_VERSION");

pub(crate)
fn start() {
    welcome();

    crate::drivers::chips::pic::remap_pic();
    crate::idt::init_idt();

    crate::memory::init();
    crate::memory::init_kernel_tracing_pages();
    #[cfg(feature="unit_tests")]
    {
        crate::memory::tests::run();
        crate::tty::print("\n\n");
    }
    crate::memory::book_kernel_allocations_space();

    crate::drivers::chips::ps2::init();

    crate::idt::enable_interrupts();

    loop { crate::halt(); }
}


fn welcome() {
    crate::tty::clear();
    crate::tty::print(KERNEL_NAME);
    crate::tty::print(" v");
    crate::tty::print(KERNEL_VERSION);
    crate::tty::print("\n");
}
