// [8042] PS/2 - PS/2 Controller

pub(crate) mod commands;
pub(crate) mod controller;
pub(crate) mod registers;


use crate::io;
use crate::drivers::chips::ps2::registers::{
    StatusRegister, read_status_register
};

pub(crate) use crate::io::{
    PS2_DATA_PORT as DATA_PORT,
    PS2_REGISTER_PORT as REGISTER_PORT,
};
pub(crate) use controller::handle_keyboard_interrupt;
pub(crate) use controller::handle_mouse_interrupt;


/// Writes 1 Byte into the input buffer of the controller
///
/// ## Warning
///
/// This function doesn't check whether the ibput buffer is actually
/// empty or not (see [`input_buffer_empty`]). Use with care.
pub(crate)
fn write_input_buffer(port:u16, byte:u8) {
    unsafe { io::out_byte(port, byte) }
}

/// Reads 1 Byte from the output buffer of the controller
///
/// ## Warning
///
/// This function doesn't check whether the output buffer is actually
/// full or not (see [`output_buffer_full`]). Use with care.
pub(crate)
fn read_output_buffer() -> u8 {
    unsafe { io::in_byte(DATA_PORT) }
}

/// Clears the output buffer of the controller
///
/// This function simply calls [`read_output_buffer()`] and discards the
/// returned value, thus clearing the `OBF` flag of the [`StatusRegister`].
/// The process is repeated until [`output_buffer_full()`] returns `false`.
pub(crate)
fn clear_output_buffer() {
    while output_buffer_full() {
        let _ = read_output_buffer();
        // TODO:
        //   Wait 30 ms before next iteration
    }
}


/// Checks if the input buffer of the controller is empty
///
/// ## Warning
///
/// Do not write into the input buffer unless it is empty.
pub(crate)
fn input_buffer_empty() -> bool {
    StatusRegister::from(read_status_register()).input_buffer_empty()
}

/// Checks if the output buffer of the controller is full
///
/// ## Warning
///
/// Do not read from the output buffer unless it is full.
pub(crate)
fn output_buffer_full() -> bool {
    StatusRegister::from(read_status_register()).output_buffer_full()
}


/// Waits until the input buffer of the controller is empty
///
/// ## Warning
///
/// This function will repeatedly check the input buffer until it becomes
/// empty, and may thus run undefinitely. Use with care.
pub(crate)
fn wait_input_buffer_empty() {
    while StatusRegister::from(read_status_register()).input_buffer_full() {
        // TODO:
        //   Wait 5 ms before next iteration
    }
}

/// Waits until the output buffer of the controller is full
///
/// ## Warning
///
/// This function will repeatedly check the output buffer until it becomes
/// full, and may thus run undefinitely. Only use when the output buffer is
/// guaranteed to be filled soon (eg. in the next 50 ms at most).
pub(crate)
fn wait_output_buffer_full() {
    while StatusRegister::from(read_status_register()).output_buffer_empty() {
        // TODO:
        //   Wait 5 ms before next iteration
    }
}


pub(crate)
fn init() {
    use crate::GetOrPanic;
    use crate::memory::{self, MemoryOwner, Init};

    crate::tty::print("[PS/2]> Initializing controller\n");

    let alloc_size = core::mem::size_of::<controller::Controller>() as u64;
    let address = memory::alloc(alloc_size, MemoryOwner::Kernel)
        .get_or_panic();
    controller::Controller::init(address);
    controller::init();
}
