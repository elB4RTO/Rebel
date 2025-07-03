use crate::io;
use crate::drivers::chips::ps2::REGISTER_PORT;
use crate::drivers::chips::ps2::commands;


const
BIT_0       : u8 = 0b00000001;
const
BIT_1       : u8 = 0b00000010;
const
BIT_2       : u8 = 0b00000100;
const
BIT_3       : u8 = 0b00001000;
const
BIT_4       : u8 = 0b00010000;
const
BIT_5       : u8 = 0b00100000;
const
BIT_6       : u8 = 0b01000000;
const
BIT_7       : u8 = 0b10000000;

const
NOT_BIT_0   : u8 = !BIT_0;
const
NOT_BIT_1   : u8 = !BIT_1;
const
NOT_BIT_2   : u8 = !BIT_2;
const
NOT_BIT_3   : u8 = !BIT_3;
const
NOT_BIT_4   : u8 = !BIT_4;
const
NOT_BIT_5   : u8 = !BIT_5;
const
NOT_BIT_6   : u8 = !BIT_6;
const
NOT_BIT_7   : u8 = !BIT_7;



/// Read Command Register
///
/// Reads Byte `0` of the internal RAM, which represents the controller
/// configuration.
///
/// ## Format
///
/// See [`CommandRegister`]
pub(in crate::drivers::chips::ps2)
fn read_command_register() -> u8 {
    commands::read_internal_ram_byte(0x00)
}

/// Write Command Register
///
/// Writes Byte `0` of the internal RAM, which represents the controller
/// configuration.
///
/// ## Format
///
/// See [`CommandRegister`]
pub(in crate::drivers::chips::ps2)
fn write_command_register(byte:u8) {
    commands::write_internal_ram_byte(0x00, byte);
}


/// Read Status Register
///
/// Reads the controller register port to retrieve the status byte
///
/// ## Format
///
/// See [`StatusRegister`]
pub(in crate::drivers::chips::ps2)
fn read_status_register() -> u8 {
    unsafe { io::in_byte(REGISTER_PORT) }
}


///
/// | Compatibility Mode | Bit 7 | Bit 6 | Bit 5 | Bit 4 | Bit 3 | Bit 2 | Bit 1 | Bit 0 |
/// | :----------------: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
/// | AT                 | ---   | XLAT  | PC    | EN    | OVR   | SYS   | ---   | INT   |
/// | PS/2               | ---   | XLAT  | EN2   | EN    | ---   | SYS   | INT2  | INT   |
#[derive(Default)]
pub(crate)
struct CommandRegister {
    byte : u8,
}

impl CommandRegister {
    /// `EN`: First I/O Port Enabled
    ///
    /// Enables/disables the communication over the first I/O port
    ///
    /// States:
    /// - **0** = **Port Enabled**: communication is enabled over the port
    /// - **1** = **Port Disabled**: communication is disabled over the port
    pub(crate)
    fn first_io_port_enabled(&self) -> bool {
        (self.byte & BIT_4) == 0x00
    }

    pub(crate)
    fn set_first_io_port_enabled(&mut self, value:bool) {
        match value {
            true  => self.byte &= NOT_BIT_4,
            false => self.byte |= BIT_4,
        }
    }

    /// `INT`: First Device Output Buffer Interrupt
    ///
    /// When set, _IRQ 1_ is generated when data is available in the output buffer
    /// of the first device.
    ///
    /// States:
    /// - **0** = **IBF Interrupt Disabled**: The output buffer must be manually checked
    ///   to assess whether new data is available
    /// - **1** = **IBF Interrupt Enabled**: An interrupt is fired whenever new data is
    ///   available in the output buffer
    pub(crate)
    fn first_io_port_interrupt_enabled(&self) -> bool {
        (self.byte & BIT_0) == BIT_0
    }

    pub(crate)
    fn set_first_io_port_interrupt_enabled(&mut self, value:bool) {
        match value {
            true  => self.byte |= BIT_0,
            false => self.byte &= NOT_BIT_0,
        }
    }

    /// `EN2`: Second I/O Port Enabled
    ///
    /// Enables/disables the communication over the second I/O port.
    ///
    /// States:
    /// - **0** = **Port Enabled**: communication is enabled over the port
    /// - **1** = **Port Disabled**: communication is disabled over the port
    pub(crate)
    fn second_io_port_enabled(&self) -> bool {
        (self.byte & BIT_5) == 0x00
    }

    pub(crate)
    fn set_second_io_port_enabled(&mut self, value:bool) {
        match value {
            true  => self.byte &= NOT_BIT_5,
            false => self.byte |= BIT_5,
        }
    }

    /// `INT2`: Second Device Output Buffer Interrupt
    ///
    /// When set, _IRQ 12_ is generated when data is available in the output buffer
    /// of the second device.
    ///
    /// States:
    /// - **0** = **IBF Interrupt Disabled**: The output buffer must be manually checked
    ///   to assess whether new data is available
    /// - **1** = **IBF Interrupt Enabled**: An interrupt is fired whenever new data is
    ///   available in the output buffer
    pub(crate)
    fn second_io_port_interrupt_enabled(&self) -> bool {
        (self.byte & BIT_1) == BIT_1
    }

    pub(crate)
    fn set_second_io_port_interrupt_enabled(&mut self, value:bool) {
        match value {
            true  => self.byte |= BIT_1,
            false => self.byte &= NOT_BIT_1,
        }
    }

    /// `SYS`: System Flag
    ///
    /// Used to manually set/clear the _SYS_ flag in the _Status Register_.
    ///
    /// States:
    /// - **0** = **Power-on value**: Tells _POST_ to perform power-on
    ///   tests/initialization
    /// - **1** = **BAT code received**: Tells _POST_ to perform "warm boot"
    ///   tests/initiailization
    pub(crate)
    fn system_flag(&self) -> bool {
        (self.byte & BIT_2) == BIT_2
    }

    pub(crate)
    fn set_system_flag(&mut self, value:bool) {
        match value {
            true  => self.byte |= BIT_2,
            false => self.byte &= NOT_BIT_2,
        }
    }

    /// `OVR`: Inhibit Override
    ///
    /// Overrides the first device's "inhibit" switch (on older motherboards).
    ///
    /// States:
    /// - **0** = **Inhibit switch enabled**: The device is inhibited if pin _P17_
    ///   is high
    /// - **1** = **Inhibit switch disabled**: The device is not inhibited even if
    ///   pin _P17_ is high
    pub(crate)
    fn inhibit_override(&self) -> bool {
        (self.byte & BIT_3) == BIT_3
    }

    pub(crate)
    fn set_inhibit_override(&mut self, value:bool) {
        match value {
            true  => self.byte |= BIT_3,
            false => self.byte &= NOT_BIT_3,
        }
    }

    /// `PC`: PC Mode (IBM)
    ///
    /// Enables/disable support for the IBM Personal Computer keyboard interface.
    /// In this mode the controller does not check parity or convert scan codes.
    ///
    /// States:
    /// - **0** = **Disabled**: The IBM Personal Computer keyboard interface is
    ///   disabled
    /// - **1** = **Enabled**: The IBM Personal Computer keyboard interface is
    ///   enabled
    pub(crate)
    fn pc_mode(&self) -> bool {
        (self.byte & BIT_5) == BIT_5
    }

    pub(crate)
    fn set_pc_mode(&mut self, value:bool) {
        match value {
            true  => self.byte |= BIT_5,
            false => self.byte &= NOT_BIT_5,
        }
    }

    /// `XLAT`: Translate Scan Codes
    ///
    /// Enables/disables translation to set 1 scan codes.
    ///
    /// States:
    /// - **0** = **Translation disabled**: Data appears in the output buffer exactly
    ///   as read from keyboard
    /// - **1** = **Translation enabled**: Scan codes are translated to _set 1_ before
    ///   appearing in the output buffer
    pub(crate)
    fn scancodes_translation(&self) -> bool {
        (self.byte & BIT_6) == BIT_6
    }

    pub(crate)
    fn set_scancodes_translation(&mut self, value:bool) {
        match value {
            true  => self.byte |= BIT_6,
            false => self.byte &= NOT_BIT_6,
        }
    }
}

impl From<u8> for CommandRegister {
    fn from(byte:u8) -> Self {
        Self { byte }
    }
}

impl Into<u8> for CommandRegister {
    fn into(self) -> u8 {
        self.byte
    }
}


///
/// | Compatibility Mode | Bit 7 | Bit 6 | Bit 5 | Bit 4 | Bit 3 | Bit 2 | Bit 1 | Bit 0 |
/// | :----------------: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
/// | AT                 | PERR  | RTO   | TTO   | INH   | A2    | SYS   | IBF   | OBF   |
/// | PS/2               | PERR  | TO    | OBF2  | INH   | A2    | SYS   | IBF   | OBF   |
#[derive(Default)]
pub(crate)
struct StatusRegister {
    byte : u8
}

impl StatusRegister {
    /// `OBF`: First Device Output Buffer Status
    ///
    /// Indicates whether data is available in the output buffer of the
    /// first device.
    ///
    /// States:
    /// - **0** = **Output buffer empty**: Don't read from the output buffer
    /// - **1** = **Output buffer full**: Ok to read from the output buffer
    pub(crate)
    fn output_buffer(&self) -> u8 {
        self.byte & BIT_0
    }

    pub(crate)
    fn output_buffer_empty(&self) -> bool {
        (self.byte & BIT_0) == 0x00
    }

    pub(crate)
    fn output_buffer_full(&self) -> bool {
        (self.byte & BIT_0) == BIT_0
    }

    /// `OBF2`: Second Device Output Buffer Status
    ///
    /// Indicates whether data is available in the output buffer of the
    /// second device.
    ///
    /// States:
    /// - **0** = **Output buffer empty**: Don't read from the output buffer
    /// - **1** = **Output buffer full**: Ok to read from the output buffer
    pub(crate)
    fn auxiliary_device_output_buffer(&self) -> bool {
        (self.byte & BIT_5) == BIT_5
    }

    /// `IBF`: Input Buffer Status
    ///
    /// Indicates whether the input buffer can receive data.
    ///
    /// States:
    /// - **0** = **Input buffer empty**: Ok to write to the input buffer
    /// - **1** = **Input buffer full**: Don't write to the input buffer
    pub(crate)
    fn input_buffer(&self) -> u8 {
        (self.byte & BIT_1) >> 1
    }

    pub(crate)
    fn input_buffer_empty(&self) -> bool {
        (self.byte & BIT_1) == 0x00
    }

    pub(crate)
    fn input_buffer_full(&self) -> bool {
        (self.byte & BIT_1) == BIT_1
    }

    /// `SYS`: System flag
    ///
    /// _POST_ reads this to determine if power-on reset, or software reset.
    ///
    /// States:
    /// - **0** = **Power-up value**: System is in power-on reset
    /// - **1** = **BAT code received**: System has already been initialized
    pub(crate)
    fn system_flag(&self) -> bool {
        (self.byte & BIT_2) == BIT_2
    }

    /// `A2`: Address line A2
    ///
    /// Used internally by the controller.
    ///
    /// States:
    /// - **0** = **A2 is 0**: Port 0x60 was last written to
    /// - **1** = **A2 is 1**: Port 0x64 was last written to
    pub(crate)
    fn a2_address_line(&self) -> bool {
        (self.byte & BIT_3) == BIT_3
    }

    /// `INH`: Inhibit Flag
    ///
    /// Indicates whether the communication with the keyboard is inhibited.
    ///
    /// States:
    /// - **0** = **Keyboard Clock is 0**: Keyboard is inhibited
    /// - **1** = **Keyboard Clock is 1**: Keyboard is not inhibited
    pub(crate)
    fn inhibit_flag(&self) -> bool {
        (self.byte & BIT_4) == BIT_4
    }

    /// `TO`: General Timout
    ///
    /// Indicates a timeout error during command write or response.
    ///
    /// States:
    /// - **0** = **No Error**: received and responded to the last command
    /// - **1** = **Timeout Error**: a timeout error occured during the last command
    pub(crate)
    fn general_timeout(&self) -> bool {
        (self.byte & BIT_6) == BIT_6
    }

    /// `TTO`: Transmit Timeout
    ///
    /// Indicates the keyboard isn't accepting input (may not be plugged in).
    ///
    /// States:
    /// - **0** = **No Error**: Keyboard accepted the last byte written to it
    /// - **1** = **Timeout error**: Keyboard didn't generate clock signals within
    ///   15 ms of "request-to-send"
    pub(crate)
    fn transmit_timeout(&self) -> bool {
        (self.byte & BIT_5) == BIT_5
    }

    /// `RTO`: Receive Timeout
    ///
    /// Indicates the keyboard didn't respond to a command (probably broke).
    ///
    /// States:
    /// - **0** = **No Error**: Keyboard responded to the last command
    /// - **1** = **Timeout error**: Keyboard didn't generate clock signals within
    ///   20 ms of command reception
    pub(crate)
    fn receive_timeout(&self) -> bool {
        (self.byte & BIT_6) == BIT_6
    }

    /// `PERR`: Parity Error
    ///
    /// Indicates a communication error with the keyboard (possibly noisy or loose
    /// connection).
    ///
    /// States:
    /// - **0** = **No Error**: Odd parity received and proper command response
    ///   recieved
    /// - **1** = **Parity Error**: Even parity received or 0xFE received as command
    ///   response
    pub(crate)
    fn parity_error(&self) -> bool {
        (self.byte & BIT_7) == BIT_7
    }
}

impl From<u8> for StatusRegister {
    fn from(byte:u8) -> Self {
        Self { byte }
    }
}
