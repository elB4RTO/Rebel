// [8259] PIC - Programmable Interrupt Controller

use crate::io;


use io::{
    PIC_MASTER_COMMAND_PORT as MASTER_COMMAND_PORT,
    PIC_MASTER_DATA_PORT as MASTER_DATA_PORT,
    PIC_SLAVE_COMMAND_PORT as SLAVE_COMMAND_PORT,
    PIC_SLAVE_DATA_PORT as SLAVE_DATA_PORT,
};


// b0=1: need 4th init step, b1=0: cascade, b2=0: address interval 8, b3=0: edge, b4=1: init
const
INITIALIZATION_BYTE         : u8 = 0b00010001;
const
MASTER_IRQ_VECTORS_OFFSET   : u8 = 32;
const
SLAVE_IRQ_VECTORS_OFFSET    : u8 = 40;
const
MASTER_IRQ_OF_SLAVE         : u8 = 0b00000100;
const
SLAVE_IDENTITY              : u8 = 0b00000010;
// b0=1: 8086 mode, b1=0: no AEOI (automatic end of interrupt), b2=0 no buffered mode for master, 3=0: no buffered mode for slave, b4=0: no fully nested mode
const
PIC_MODE_BYTE               : u8 = 0b00000001;

// mask all IRQs except IRQ0 (timer), IRQ1 (keyboard), IRQ2 (mouse) and IRQ7 (spurious)
const
INITIAL_MASTER_IRQ_MASKS    : u8 = 0b01111000;
// mask all IRQs except IRQ12 (mouse) and IRQ15 (spurious)
const
INITIAL_SLAVE_IRQ_MASKS     : u8 = 0b01101111;

// b5=1: non-specific end of intettupt
const
EOI         : u8 = 0b00100000;

// b0-1=1: reading the ISR register, b4=1: read the ISR register
const
READ_IRR    : u8 = 0b00001010;
// b0-1=1: reading the ISR register, b4=1: read the ISR register
const
READ_ISR    : u8 = 0b00001011;


/// Informs the processor the ISR has been handled
pub(crate)
fn aknowledge_interrupt() {
    unsafe { io::out_byte(MASTER_COMMAND_PORT, EOI); }
}

/// Informs the processor the IRQ has been handled
pub(crate)
fn aknowledge_interrupt_request(irq:u64) {
    if irq > 7 {
        unsafe { io::out_byte(SLAVE_COMMAND_PORT, EOI); }
    }
    unsafe { io::out_byte(MASTER_COMMAND_PORT, EOI); }
}


/// Reads the _Interrupt Request Register_ of the master PIC
///
/// The IRR tells which IRQs have been raised.
pub(crate)
fn read_master_irr() -> u8 {
    unsafe {
        io::out_byte(MASTER_COMMAND_PORT, READ_IRR);
        io::in_byte(MASTER_COMMAND_PORT)
    }
}

/// Reads the ISR _Interrupt Request Register_ of the slave PIC
///
/// The IRR tells which IRQs have been raised.
pub(crate)
fn read_slave_irr() -> u8 {
    unsafe {
        io::out_byte(SLAVE_COMMAND_PORT, READ_IRR);
        io::in_byte(SLAVE_COMMAND_PORT)
    }
}


/// Reads the _In-Service Register_ of the master PIC
///
/// The ISR tells which IRQs are beinge serviced.
pub(crate)
fn read_master_isr() -> u8 {
    unsafe {
        io::out_byte(MASTER_COMMAND_PORT, READ_ISR);
        io::in_byte(MASTER_COMMAND_PORT)
    }
}

/// Reads the ISR _In-Service Register_ of the slave PIC
///
/// The ISR tells which IRQs are beinge serviced.
pub(crate)
fn read_slave_isr() -> u8 {
    unsafe {
        io::out_byte(SLAVE_COMMAND_PORT, READ_ISR);
        io::in_byte(SLAVE_COMMAND_PORT)
    }
}


pub(crate)
fn mask_irq(irq:u8) {
    let (port, bit) = match irq < 8 {
        true => (MASTER_DATA_PORT, 1 << irq),
        false => (SLAVE_DATA_PORT, 1 << (irq - 8)),
    };
    unsafe {
        let mask = io::in_byte(port);
        io::out_byte(port, mask | bit);
    }
}

pub(crate)
fn unmask_irq(irq:u8) {
    let (port, bit) = match irq < 8 {
        true => (MASTER_DATA_PORT, 1 << irq),
        false => (SLAVE_DATA_PORT, 1 << (irq - 8)),
    };
    unsafe {
        let mask = io::in_byte(port);
        io::out_byte(port, mask & !bit);
    }
}


pub(crate)
fn remap_pic() {
    crate::tty::print("[PIC]> Remapping controller\n");

    unsafe {
        // step 1
        // start the initialization sequence
        io::out_byte(MASTER_COMMAND_PORT, INITIALIZATION_BYTE);
        io::out_byte(SLAVE_COMMAND_PORT, INITIALIZATION_BYTE);

        // step 2
        // re-map IRQ vectors offsets
        io::out_byte(MASTER_DATA_PORT, MASTER_IRQ_VECTORS_OFFSET);
        io::out_byte(SLAVE_DATA_PORT, SLAVE_IRQ_VECTORS_OFFSET);

        // step 3
        // attach slave to master via IRQ
        io::out_byte(MASTER_DATA_PORT, MASTER_IRQ_OF_SLAVE);
        // set slave on cascade
        io::out_byte(SLAVE_DATA_PORT, SLAVE_IDENTITY);

        // step 4
        // set PIC mode
        io::out_byte(MASTER_DATA_PORT, PIC_MODE_BYTE);
        io::out_byte(SLAVE_DATA_PORT, PIC_MODE_BYTE);

        // set initial IRQs masks
        io::out_byte(MASTER_DATA_PORT, 0);
        io::out_byte(SLAVE_DATA_PORT, 0);
    }
}
