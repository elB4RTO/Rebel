mod idt;
mod isr;
mod syscall;


pub(crate) use idt::init_idt;


pub(crate)
fn enable_interrupts() {
    unsafe { idt::enable_interrupts(); }
}

pub(crate)
fn disable_interrupts() {
    unsafe { idt::disable_interrupts(); }
}
