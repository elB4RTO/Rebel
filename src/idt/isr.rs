// ISR - Interrupt Service Routine

use crate::drivers::chips::pic::aknowledge_interrupt;
use crate::idt::idt::InterruptFrame;
use crate::idt::syscall::system_call;


pub(in crate::idt)
fn unexpected_isr(_:&mut InterruptFrame) {
    // TODO:
    //   Log the unexpected interrupt
    aknowledge_interrupt();
}

/// Divide Error
pub(in crate::idt)
fn isr0(_:&mut InterruptFrame) {
    aknowledge_interrupt();
}

/// Debug Exception
pub(in crate::idt)
fn isr1(_:&mut InterruptFrame) {
    aknowledge_interrupt();
}

/// NMI Interrupt
pub(in crate::idt)
fn isr2(_:&mut InterruptFrame) {
    aknowledge_interrupt();
}

/// Breakpoint
pub(in crate::idt)
fn isr3(_:&mut InterruptFrame) {
    aknowledge_interrupt();
}

/// Overflow
pub(in crate::idt)
fn isr4(_:&mut InterruptFrame) {
    aknowledge_interrupt();
}

/// BOUND Range Exceeded
pub(in crate::idt)
fn isr5(_:&mut InterruptFrame) {
    aknowledge_interrupt();
}

/// Undefined Opcode
pub(in crate::idt)
fn isr6(_:&mut InterruptFrame) {
    aknowledge_interrupt();
}

/// No Math Coprocessor
pub(in crate::idt)
fn isr7(_:&mut InterruptFrame) {
    aknowledge_interrupt();
}

/// Double Fault
pub(in crate::idt)
fn isr8(_:&mut InterruptFrame) {
    aknowledge_interrupt();
}

/// Invalid TSS
pub(in crate::idt)
fn isr10(_:&mut InterruptFrame) {
    aknowledge_interrupt();
}

/// Segment Not Present
pub(in crate::idt)
fn isr11(_:&mut InterruptFrame) {
    aknowledge_interrupt();
}

/// Stack-Segment Fault
pub(in crate::idt)
fn isr12(_:&mut InterruptFrame) {
    aknowledge_interrupt();
}

/// General Protection
pub(in crate::idt)
fn isr13(_:&mut InterruptFrame) {
    aknowledge_interrupt();
}

/// Page Fault
pub(in crate::idt)
fn isr14(_:&mut InterruptFrame) {
    aknowledge_interrupt();
}

/// x87 FPU Floating-Point Error
pub(in crate::idt)
fn isr16(_:&mut InterruptFrame) {
    aknowledge_interrupt();
}

/// Alignment Check
pub(in crate::idt)
fn isr17(_:&mut InterruptFrame) {
    aknowledge_interrupt();
}

/// Machine Check
pub(in crate::idt)
fn isr18(_:&mut InterruptFrame) {
    aknowledge_interrupt();
}

/// SIMD Floating-Point Exception
pub(in crate::idt)
fn isr19(_:&mut InterruptFrame) {
    aknowledge_interrupt();
}

/// Virtualization Exception
pub(in crate::idt)
fn isr20(_:&mut InterruptFrame) {
    aknowledge_interrupt();
}

/// Control Protection Exception
pub(in crate::idt)
fn isr21(_:&mut InterruptFrame) {
    aknowledge_interrupt();
}

/// Software Interrupt
pub(in crate::idt)
fn isr128(stack_frame:&mut InterruptFrame) {
    system_call(stack_frame);
    aknowledge_interrupt();
}
