/// Interrupt Descriptor Table
mod idt;
/// Interrupt Request
mod irq;
/// Interrupt Service Routine
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
