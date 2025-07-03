use crate::drivers::chips::ps2::*;


/// PS/2 controller commands bytes
///
/// These commands shall be written to port 0x64
mod command {
    /// read internal ram byte
    pub(super) const
    READ_INTERNAL_RAM_BASE      : u8 = 0x20;
    /// write internal ram byte
    pub(super) const
    WRITE_INTERNAL_RAM_BASE     : u8 = 0x60;
    /// query the firmware version number
    pub(super) const
    READ_FIRMWARE_VERSION       : u8 = 0xA1;
    /// check if a password is set (returns 0xFA if set, otherwise 0xF1)
    pub(super) const
    HAS_PASSWORD                : u8 = 0xA4;
    /// change the password (by sending a null-terminated string of scan codes as this command's parameter)
    pub(super) const
    SET_PASSWORD                : u8 = 0xA5;
    /// enables the current password
    pub(super) const
    ENABLE_PASSWORD             : u8 = 0xA6;
    /// (PS/2 mode only) disable the communication on the second I/O port
    pub(super) const
    DISABLE_SECOND_IO_PORT      : u8 = 0xA7;
    /// (PS/2 mode only) enable the communication on the second I/O port
    pub(super) const
    ENABLE_SECOND_IO_PORT       : u8 = 0xA8;
    /// (PS/2 mode only) test the second I/O port
    pub(super) const
    TEST_SECOND_IO_PORT         : u8 = 0xA9;
    /// test the controller
    pub(super) const
    TEST_CONTROLLER             : u8 = 0xAA;
    /// test the first I/O port
    pub(super) const
    TEST_FIRST_IO_PORT          : u8 = 0xAB;
    /// dump diagnostic data
    pub(super) const
    DIAGNOSTIC_DUMP             : u8 = 0xAC;
    /// disable the communication on the first I/O port
    pub(super) const
    DISABLE_FIRST_IO_PORT       : u8 = 0xAD;
    /// enable the communication on the first I/O port
    pub(super) const
    ENABLE_FIRST_IO_PORT        : u8 = 0xAE;
    /// query the version
    pub(super) const
    READ_VERSION                : u8 = 0xAF;
    /// read data from the controller input port
    pub(super) const
    READ_INPUT_PORT             : u8 = 0xC0;
    /// copy the controller input port's low nibble (bits 0~3) to the status register (bits 4~7)
    pub(super) const
    POLL_INPUT_PORT_LSN         : u8 = 0xC1;
    /// copy the controller input port's high nibble (bits 4~7) to the status register (bits 4~7)
    pub(super) const
    POLL_INPUT_PORT_MSN         : u8 = 0xC2;
    /// read data from the controller output port
    pub(super) const
    READ_OUTPUT_PORT            : u8 = 0xD0;
    /// write data to the controller output port
    pub(super) const
    WRITE_OUTPUT_PORT           : u8 = 0xD1;
    /// write data to the output buffer of the first device
    pub(super) const
    WRITE_FIRST_OUTPUT_BUFFER   : u8 = 0xD2;
    /// write data to the input buffer as if received from auxiliary device
    pub(super) const
    WRITE_SECOND_OUTPUT_BUFFER  : u8 = 0xD3;
    /// send data to the auxillary device
    pub(super) const
    WRITE_SECOND_INPUT_BUFFER   : u8 = 0xD4;
    /// read data from the test port
    pub(super) const
    READ_TEST_PORT              : u8 = 0xE0;
    /// pulse output port
    pub(super) const
    PULSE_OUTPUT_PORT_BASE      : u8 = 0xF0;
}

/// PS/2 controller responses bytes
///
/// These commands shall be read from port 0x60
pub(in crate::drivers::chips::ps2)
mod response {
    /// Interface test: no error
    pub(in crate::drivers::chips::ps2) const
    IO_PORT_TEST_NO_ERROR       : u8 = 0x00;
    /// Interface test: keyboard clock line is stuck low
    pub(in crate::drivers::chips::ps2) const
    IO_PORT_TEST_CLOCK_LOW      : u8 = 0x01;
    /// Interface test: keyboard clock line is stuck high
    pub(in crate::drivers::chips::ps2) const
    IO_PORT_TEST_CLOCK_HIGH     : u8 = 0x02;
    /// Interface test: keyboard data line is stuck low
    pub(in crate::drivers::chips::ps2) const
    IO_PORT_TEST_DATA_LOW       : u8 = 0x03;
    /// Interface test: keyboard data line is stuck high
    pub(in crate::drivers::chips::ps2) const
    IO_PORT_TEST_DATA_HIGH      : u8 = 0x04;
    /// Self test passed
    pub(in crate::drivers::chips::ps2) const
    SELF_TEST_OK                : u8 = 0x55;
    /// No password installed
    pub(in crate::drivers::chips::ps2) const
    PASSWORD_NOT_INSTALLED      : u8 = 0xF1;
    /// Password installed
    pub(in crate::drivers::chips::ps2) const
    PASSWORD_INSTALLED          : u8 = 0xFA;
    /// Self test failed
    pub(in crate::drivers::chips::ps2) const
    SELF_TEST_ERR               : u8 = 0xFC;
}


/// Read Internal RAM
///
/// Reads the Byte at `idx` of the internal RAM.
///
/// ## Range
///
/// Index from `0x00` to `0x1F`.
pub(in crate::drivers::chips::ps2)
fn read_internal_ram_byte(idx:u8) -> u8 {
    let cmd = command::READ_INTERNAL_RAM_BASE | (idx & 0x1F);
    wait_input_buffer_empty();
    write_input_buffer(REGISTER_PORT, cmd);
    wait_output_buffer_full();
    read_output_buffer()
}

/// Write Internal RAM
///
/// Writes the Byte at `idx` of the internal RAM.
///
/// ## Range
///
/// Index from `0x00` to `0x1F`.
pub(in crate::drivers::chips::ps2)
fn write_internal_ram_byte(idx:u8, byte:u8) {
    let cmd = command::WRITE_INTERNAL_RAM_BASE | (idx & 0x1F);
    wait_input_buffer_empty();
    write_input_buffer(REGISTER_PORT, cmd);
    wait_input_buffer_empty();
    write_input_buffer(DATA_PORT, byte);
}

/// Password Installed Test
///
/// ## Response
///
/// - `0xFA` = password installed
/// - `0xF1` = no password
pub(in crate::drivers::chips::ps2)
fn has_password() -> u8 {
    wait_input_buffer_empty();
    write_input_buffer(REGISTER_PORT, command::HAS_PASSWORD);
    wait_output_buffer_full();
    read_output_buffer()
}

/// Install Password
///
/// ## Format
///
/// A null-terminated sequence of bytes.
pub(in crate::drivers::chips::ps2)
fn set_password(psw:&[u8]) {
    wait_input_buffer_empty();
    write_input_buffer(REGISTER_PORT, command::SET_PASSWORD);
    for byte in psw {
        wait_input_buffer_empty();
        write_input_buffer(DATA_PORT, *byte);
    }
}

/// Enable Password
///
/// ## Note
///
/// Only works if a password is already installed.
pub(in crate::drivers::chips::ps2)
fn enable_password() {
    wait_input_buffer_empty();
    write_input_buffer(REGISTER_PORT, command::ENABLE_PASSWORD);
}

/// Disable Second I/O Port
///
/// Sets bit `5` of the command register, thus disabling the communication
/// over the second I/O port by driving the clock line low.
/// This port is usually associated with the mouse.
pub(in crate::drivers::chips::ps2)
fn disable_second_io_port() {
    wait_input_buffer_empty();
    write_input_buffer(REGISTER_PORT, command::DISABLE_SECOND_IO_PORT);
}

/// Enable Second I/O Port
///
/// Clears bit `5` of the command register, thus disabling the communication
/// over the second I/O port.
/// This port is usually associated with the mouse.
pub(in crate::drivers::chips::ps2)
fn enable_second_io_port() {
    wait_input_buffer_empty();
    write_input_buffer(REGISTER_PORT, command::ENABLE_SECOND_IO_PORT);
}

/// Second I/O Port Test
///
/// Tests the clock and data lines of the second I/O port.
/// This port is usually associated with the mouse.
///
/// ## Returns
///
/// - `0x00` = no error
/// - `0x01` = clock line is stuck low
/// - `0x02` = clock line is stuck high
/// - `0x03` = data line is stuck low
/// - `0x04` = data line is stuck high
pub(in crate::drivers::chips::ps2)
fn test_second_io_port() -> u8 {
    wait_input_buffer_empty();
    write_input_buffer(REGISTER_PORT, command::TEST_SECOND_IO_PORT);
    wait_output_buffer_full();
    read_output_buffer()
}

/// Controller Test
///
/// ## Returns
///
/// - `0x55` = test passed
/// - `0xFC` = test failed
pub(in crate::drivers::chips::ps2)
fn self_test() -> u8 {
    wait_input_buffer_empty();
    write_input_buffer(REGISTER_PORT, command::TEST_CONTROLLER);
    wait_output_buffer_full();
    read_output_buffer()
}

/// First I/O Port Test
///
/// Tests the clock and data lines of the first I/O port.
/// This port is usually associated with the keyboard.
///
/// ## Returns
///
/// - `0x00` = no error
/// - `0x01` = clock line is stuck low
/// - `0x02` = clock line is stuck high
/// - `0x03` = data line is stuck low
/// - `0x04` = data line is stuck high
pub(in crate::drivers::chips::ps2)
fn test_first_io_port() -> u8 {
    wait_input_buffer_empty();
    write_input_buffer(REGISTER_PORT, command::TEST_FIRST_IO_PORT);
    wait_output_buffer_full();
    read_output_buffer()
}

/// Diagnostic Dump
///
/// ## Returns
///
/// Returns 16 bytes of the internal RAM, the input port state, the output
/// port state and the program status word.
pub(in crate::drivers::chips::ps2)
fn diagnostic_dump() -> [u8; 20] {
    wait_input_buffer_empty();
    write_input_buffer(REGISTER_PORT, command::DIAGNOSTIC_DUMP);
    let mut dump = [0_u8; 20];
    for i in 0..20 {
        wait_output_buffer_full();
        dump[i] = read_output_buffer();
    }
    dump
}

/// Disable First I/O Port
///
/// Sets bit `4` of the command register, thus disabling the communication
/// over the first I/O port by driving the clock line low.
/// This port is usually associated with the keyboard.
pub(in crate::drivers::chips::ps2)
fn disable_first_io_port() {
    wait_input_buffer_empty();
    write_input_buffer(REGISTER_PORT, command::DISABLE_FIRST_IO_PORT);
}

/// Enable First I/O Port
///
/// Clears bit `4` of command register, thus enabling the communication
/// over the first I/O port.
/// This port is usually associated with the keyboard.
pub(in crate::drivers::chips::ps2)
fn enable_first_io_port() {
    wait_input_buffer_empty();
    write_input_buffer(REGISTER_PORT, command::ENABLE_FIRST_IO_PORT);
}

/// Read Controller Input Port
///
/// Read the controller input port (which is inaccessible to the data bus).
///
/// ## Format
///
/// #### AT compatibility mode
///
/// | Bit | Description                                   |
/// | :-: | :-------------------------------------------- |
/// |  0  | _undefined_                                   |
/// |  1  | _undefined_                                   |
/// |  2  | _undefined_                                   |
/// |  3  | _undefined_                                   |
/// |  4  | - **1** = enable 2nd 256K of motherboard RAM  |
/// |     | - **0** = disable 2nd 256K of motherboard RAM |
/// |  5  | - **1** = manufacturing jumper not installed  |
/// |     | - **0** = manufacturing jumper installed      |
/// |  6  | - **1** = primary display is MDA              |
/// !     | - **0** = primary display is CGA              |
/// |  7  | - **1** = keyboard not inhibited              |
/// |     | - **0** = keyboard inhibited                  |
///
/// #### PS/2 compatibility mode
///
/// _???_
pub(in crate::drivers::chips::ps2)
fn read_input_port() -> u8 {
    wait_input_buffer_empty();
    write_input_buffer(REGISTER_PORT, command::READ_INPUT_PORT);
    wait_output_buffer_full();
    read_output_buffer()
}

/// Poll Controller Input Port Least Significant Nibble
///
/// Bits `0`~`3` of the input port placed in the status register Bits `4`~`7`.
pub(in crate::drivers::chips::ps2)
fn poll_input_port_lsn() {
    wait_input_buffer_empty();
    write_input_buffer(REGISTER_PORT, command::POLL_INPUT_PORT_LSN);
}

/// Poll Controller Input Port Most Significant Nibble
///
/// Bits `4`~`7` of the input port placed in the status register Bits `4`~`7`.
pub(in crate::drivers::chips::ps2)
fn poll_input_port_msn() {
    wait_input_buffer_empty();
    write_input_buffer(REGISTER_PORT, command::POLL_INPUT_PORT_MSN);
}

/// Read Controller Output Port
///
/// Reads the controller output port (which is inaccessible to the data bus)
///
/// ## Note
///
/// The output register should be empty before executing this command.
///
/// ## Format
///
/// #### AT compatibility mode
///
/// | Bit | Description                 |
/// | :-: | :-------------------------- |
/// |  0  | system reset (always 1)     |
/// |  1  | gate A20                    |
/// |  2  | _undefined_                 |
/// |  3  | _undefined_                 |
/// |  4  | keyboard buffer full        |
/// |  5  | keyboard input buffer empty |
/// |  6  | keyboard clock (output)     |
/// |  7  | keyboard data (output)      |
///
/// #### PS/2 compatibility mode
///
/// | Bit | Description                                       |
/// | :-: | :------------------------------------------------ |
/// |  0  | system reset (always 1)                           |
/// |  1  | gate A20                                          |
/// |  2  | second I/O port clock (output)                    |
/// |  3  | second I/O port data (output)                     |
/// |  4  | output buffer full with data from first I/O port  |
/// |  5  | output buffer full with data from second I/O port |
/// |  6  | first I/O port clock (output)                     |
/// |  7  | first I/O port data (output)                      |
pub(in crate::drivers::chips::ps2)
fn read_output_port() -> u8 {
    wait_input_buffer_empty();
    write_input_buffer(REGISTER_PORT, command::READ_OUTPUT_PORT);
    wait_output_buffer_full();
    read_output_buffer()
}

/// Write Controller Output Port
///
/// Writes 1 Byte in the controller output port (which is inaccessible to the data bus)
///
/// ## Format
///
/// See [`read_output_port()`].
pub(in crate::drivers::chips::ps2)
fn write_output_port(byte:u8) {
    wait_input_buffer_empty();
    write_input_buffer(REGISTER_PORT, command::WRITE_OUTPUT_PORT);
    wait_input_buffer_empty();
    write_input_buffer(DATA_PORT, byte);
}

/// Write First Device Output Buffer
///
/// Writes 1 Byte to the output register of the first device, as if
/// it was written by the device. Triggers an interrupt if interrupts
/// are enabled.
/// This interface is usually associated with the keyboard.
pub(in crate::drivers::chips::ps2)
fn write_first_device_output_buffer(byte:u8) {
    wait_input_buffer_empty();
    write_input_buffer(REGISTER_PORT, command::WRITE_FIRST_OUTPUT_BUFFER);
    wait_input_buffer_empty();
    write_input_buffer(DATA_PORT, byte);
}

/// Write Second Device Output Buffer
///
/// Writes 1 Byte to the output register of the second device, as if
/// it was written by the device. Triggers an interrupt if interrupts
/// are enabled.
/// This interface is usually associated with the mouse.
pub(in crate::drivers::chips::ps2)
fn write_second_device_output_buffer(byte:u8) {
    wait_input_buffer_empty();
    write_input_buffer(REGISTER_PORT, command::WRITE_SECOND_OUTPUT_BUFFER);
    wait_input_buffer_empty();
    write_input_buffer(DATA_PORT, byte);
}

/// Write Second Device Input Buffer
///
/// Writes 1 Byte to the input register of the second device.
/// This interface is usually associated with the mouse.
pub(in crate::drivers)
fn write_second_device_input_buffer(byte:u8) {
    wait_input_buffer_empty();
    write_input_buffer(REGISTER_PORT, command::WRITE_SECOND_INPUT_BUFFER);
    wait_input_buffer_empty();
    write_input_buffer(DATA_PORT, byte);
}

/// Read Test Inputs
///
/// Reads the controller T0 and T1 inputs
///
/// ## Format
///
/// | Bit | Description    |
/// | :-: | :------------- |
/// |  0  | keyboard clock |
/// |  1  | keyboard data  |
pub(in crate::drivers::chips::ps2)
fn read_test_inputs() -> u8 {
    wait_input_buffer_empty();
    write_input_buffer(REGISTER_PORT, command::READ_TEST_PORT);
    wait_output_buffer_full();
    read_output_buffer()
}

/// Pulse Output Port
///
/// Pulses the output port low for 6 µs.
///
/// ## Format
///
/// Bits `0`~`3` indicate which bits of the port should be pulsed:
/// - **0** = pulse
/// - **1** = don't pulse
///
/// ## Range
///
/// Ports from `0x00` to `0x0F`.
///
/// ## Note
///
/// Pulsing bit `0` results in CPU reset since it is connected to system
/// reset line.
pub(in crate::drivers::chips::ps2)
fn pulse_output_port(port:u8, byte:u8) {
    let cmd = command::PULSE_OUTPUT_PORT_BASE | (port & 0x0F);
    wait_input_buffer_empty();
    write_input_buffer(REGISTER_PORT, cmd);
    wait_input_buffer_empty();
    write_input_buffer(DATA_PORT, byte);
}
