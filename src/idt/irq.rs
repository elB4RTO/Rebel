// IRQ - Interrupt Request

use crate::drivers::chips::pic::{
    aknowledge_interrupt_request, read_master_isr, read_slave_isr
};
use crate::idt::idt::InterruptFrame;


/// Timer Interrupt (MASTER)
pub(in crate::idt)
fn irq0(_:&mut InterruptFrame) {
    aknowledge_interrupt_request(0);
    // try to pass CPU resources to a waiting process
}

/// PS/2 Keyboard Interrupt (MASTER)
pub(in crate::idt)
fn irq1(_:&mut InterruptFrame) {
    aknowledge_interrupt_request(1);
}

/// Spurious Interrupt (MASTER)
pub(in crate::idt)
fn irq7(_:&mut InterruptFrame) {
    let isr_value = read_master_isr();
    if (isr_value & 0b10000000) != 0 {
        // if the seventh bit is 1 then this is a real interrupt,
        // otherwise this is a spurious interrupt
        aknowledge_interrupt_request(7);
    }
}

/// PS/2 Mouse Interrupt (SLAVE)
pub(in crate::idt)
fn irq12(_:&mut InterruptFrame) {
    aknowledge_interrupt_request(12);
}

/// Spurious Interrupt (SLAVE)
pub(in crate::idt)
fn irq15(_:&mut InterruptFrame) {
    let isr_value = read_slave_isr();
    if (isr_value & 0b10000000) != 0 {
        // if the seventh bit is 1 then this is a real interrupt,
        // otherwise this is a spurious interrupt
        aknowledge_interrupt_request(15);
    }
    // the master PIC has no knowledge about the slave register,
    // so it needs to be aknowledged in any case
    aknowledge_interrupt_request(7);
}
