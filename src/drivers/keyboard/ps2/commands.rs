use crate::drivers::chips::ps2::{
    DATA_PORT, wait_input_buffer_empty, wait_output_buffer_full,
    write_input_buffer, read_output_buffer
};


pub(in crate::drivers::keyboard::ps2) const
SCANCODE_SET_1  : u8 = 0x43;
pub(in crate::drivers::keyboard::ps2) const
SCANCODE_SET_2  : u8 = 0x41;
pub(in crate::drivers::keyboard::ps2) const
SCANCODE_SET_3  : u8 = 0x3F;


/// Keyboard interface commands
///
/// These commands shall be written to port 0x60
mod command {
    /// Change leds state
    pub(super) const
    LEDS_STATE          : u8 = 0xE0;
    /// Diagnostics
    pub(super) const
    ECHO                : u8 = 0xEE;
    /// Query/change scancode set
    pub(super) const
    SCANCODE_SET        : u8 = 0xF0;
    /// Query the device ID
    pub(super) const
    DEVICE_ID           : u8 = 0xF2;
    /// Change keypress repeat rate and delay
    pub(super) const
    TYPEMATIC_OPTIONS   : u8 = 0xF3;
    /// Enable sending scancodes
    pub(super) const
    ENABLE_SCANCODES    : u8 = 0xF4;
    /// Disable sending scancodes
    pub(super) const
    DISABLE_SCANCODES   : u8 = 0xF5;
    /// Reset to default parameters
    pub(super) const
    RESET_DEFAULTS      : u8 = 0xF6;
    /// (set 3) Set all keys to typematic
    pub(super) const
    SET_ALL_T           : u8 = 0xF7;
    /// (set 3) Set all keys to make/break
    pub(super) const
    SET_ALL_MB          : u8 = 0xF8;
    /// (set 3) Set all keys to make
    pub(super) const
    SET_ALL_M           : u8 = 0xF9;
    /// (set 3) Set all keys to typematic make/break
    pub(super) const
    SET_ALL_TMB         : u8 = 0xFA;
    /// (set 3) Set a signle key to typematic
    pub(super) const
    SET_KEY_T           : u8 = 0xFB;
    /// (set 3) Set a signle key to make/break
    pub(super) const
    SET_KEY_MB          : u8 = 0xFC;
    /// (set 3) Set a signle key to make
    pub(super) const
    SET_KEY_M           : u8 = 0xFD;
    /// Resend the last byte
    pub(super) const
    RESEND              : u8 = 0xFE;
    /// Reset and perform self-testing
    pub(super) const
    RESET_AND_TEST      : u8 = 0xFF;
}

pub(in crate::drivers::keyboard::ps2)
mod subcommand {
    pub(super) const
    READ_SCANCODE_SET   : u8 = 0x00;
    pub(in crate::drivers::keyboard::ps2) const
    WRITE_SCANCODE_SET1 : u8 = 0x01;
    pub(in crate::drivers::keyboard::ps2) const
    WRITE_SCANCODE_SET2 : u8 = 0x02;
    pub(in crate::drivers::keyboard::ps2) const
    WRITE_SCANCODE_SET3 : u8 = 0x03;
}

pub(in crate::drivers::keyboard::ps2)
mod response {
    /// Key detection error or internal buffer overrun
    pub(in crate::drivers::keyboard::ps2) const
    SET_1_ERROR         : u8 = 0x00;
    /// Self test passed
    pub(in crate::drivers::keyboard::ps2) const
    TEST_OK             : u8 = 0xAA;
    /// Response to ECHO
    pub(in crate::drivers::keyboard::ps2) const
    ECHO                : u8 = 0xEE;
    /// Command acknowledged
    pub(in crate::drivers::keyboard::ps2) const
    ACK                 : u8 = 0xFA;
    /// Self test failed
    pub(in crate::drivers::keyboard::ps2) const
    TEST_ERR1           : u8 = 0xFC;
    pub(in crate::drivers::keyboard::ps2) const
    TEST_ERR2           : u8 = 0xFD;
    /// Resend last command
    pub(in crate::drivers::keyboard::ps2) const
    RESEND              : u8 = 0xFE;
    /// Key detection error or internal buffer overrun
    pub(in crate::drivers::keyboard::ps2) const
    SET_2_SET_3_ERROR   : u8 = 0xFF;
}


/// Set Mode Indicators
///
/// When the option byte is received, scanning is resumed if it was enabled.
/// If another command is received instead of the option byte (high bit set),
/// this command is terminated.
/// Hardware defaults to the indicators turned off.
///
/// ## Format
///
/// - **0** = off
/// - **1** = on
///
/// | Bit | Description            |
/// | :-: | :--------------------- |
/// |  0  | Scroll-Lock indicator  |
/// |  1  | Num-Lock indicator     |
/// |  2  | Caps-Lock indicator    |
/// |  3  | _reserved (must be 0)_ |
/// |  4  | _reserved (must be 0)_ |
/// |  5  | _reserved (must be 0)_ |
/// |  6  | _reserved (must be 0)_ |
/// |  7  | _reserved (must be 0)_ |
pub(in crate::drivers::keyboard::ps2)
fn write_leds_state(byte:u8) -> Result<(), u8> {
    wait_input_buffer_empty();
    write_input_buffer(DATA_PORT, command::LEDS_STATE);
    wait_output_buffer_full();
    match read_output_buffer() {
        response::ACK => (),
        resp => return Err(resp),
    }
    wait_input_buffer_empty();
    write_input_buffer(DATA_PORT, byte);
    wait_output_buffer_full();
    match read_output_buffer() {
        response::ACK => Ok(()),
        resp => Err(resp),
    }
}

/// Diagnostic Echo
///
/// Keyboard echoes back to the system
pub(in crate::drivers::keyboard::ps2)
fn send_echo() -> Result<u8, u8> {
    wait_input_buffer_empty();
    write_input_buffer(DATA_PORT, command::ECHO);
    wait_output_buffer_full();
    match read_output_buffer() {
        response::ECHO => Ok(response::ECHO),
        resp => Err(resp),
    }
}

/// Read Scan Code Set
///
/// Reads the Scan Code Set in use
pub(in crate::drivers::keyboard::ps2)
fn read_scancode_set() -> Result<u8, u8> {
    wait_input_buffer_empty();
    write_input_buffer(DATA_PORT, command::SCANCODE_SET);
    wait_output_buffer_full();
    match read_output_buffer() {
        response::ACK => (),
        resp => return Err(resp),
    }
    wait_input_buffer_empty();
    write_input_buffer(DATA_PORT, subcommand::READ_SCANCODE_SET);
    wait_output_buffer_full();
    match read_output_buffer() {
        response::ACK => (),
        resp => return Err(resp),
    }
    wait_output_buffer_full();
    Ok(read_output_buffer())
}

/// Select Scan Code Set
///
/// Instructs keyboard to use one of the three make/break scan code sets
///
/// ## Format
///
/// - `0x01`: select scan code set 1 (used on PC & XT)
/// - `0x02`: select scan code set 2
/// - `0x03`: select scan code set 3
///
/// ## Default
///
/// Defaults to Scan Code Set 2
pub(in crate::drivers::keyboard::ps2)
fn write_scancode_set(byte:u8) -> Result<(), u8> {
    wait_input_buffer_empty();
    write_input_buffer(DATA_PORT, command::SCANCODE_SET);
    wait_output_buffer_full();
    match read_output_buffer() {
        response::ACK => (),
        resp => return Err(resp),
    }
    wait_input_buffer_empty();
    write_input_buffer(DATA_PORT, byte);
    wait_output_buffer_full();
    match read_output_buffer() {
        response::ACK => Ok(()),
        resp => Err(resp),
    }
}

/// Read Keyboard ID
///
/// PS/2 keyboards respond with a two byte keyboard ID of `0x83AB`
pub(in crate::drivers::keyboard::ps2)
fn read_device_id() -> Result<[u8; 2], u8> {
    wait_input_buffer_empty();
    write_input_buffer(DATA_PORT, command::DEVICE_ID);
    wait_output_buffer_full();
    match read_output_buffer() {
        response::ACK => (),
        resp => return Err(resp),
    }
    let mut bytes = [0_u8; 2];
    wait_output_buffer_full();
    bytes[0] = read_output_buffer();
    wait_output_buffer_full();
    bytes[1] = read_output_buffer();
    Ok(bytes)
}

/// Set Typematic Rate/Delay
///
/// Upon receipt of the rate/delay byte, the scanning continues if
/// scanning was enabled.
/// If a command byte (high bit set) is received, this command is cancelled.
///
/// ## Format
///
/// | Bit | Description                          |
/// | :-: | :----------------------------------- |
/// |  0  | typematic rate (A in period formula) |
/// |  1  | typematic rate (A in period formula) |
/// |  2  | typematic rate (A in period formula) |
/// |  3  | typematic rate (B in period formula) |
/// |  4  | typematic rate (B in period formula) |
/// |  5  | typematic delay                      |
/// |  6  | typematic delay                      |
/// |  7  | _reserved (must be zero)_            |
///
/// ## Formula
///
/// - **delay** (ms) = `(rate + 1) * 250`
/// - **rate** (hz) = `(8 + A) * (2 ** B) * 4.17`
///
/// ## Default
///
/// Defaults to 10.9 hz rate and a 500 ms delay
pub(in crate::drivers::keyboard::ps2)
fn write_typematic_options(byte:u8) -> Result<(), u8> {
    wait_input_buffer_empty();
    write_input_buffer(DATA_PORT, command::TYPEMATIC_OPTIONS);
    wait_output_buffer_full();
    match read_output_buffer() {
        response::ACK => (),
        resp => return Err(resp),
    }
    wait_input_buffer_empty();
    write_input_buffer(DATA_PORT, byte);
    wait_output_buffer_full();
    match read_output_buffer() {
        response::ACK => Ok(()),
        resp => Err(resp),
    }
}

/// Enable Keyboard
///
/// Causes the keyboard to clear its output buffer and last typematic key.
/// The keyboard then begins scanning.
pub(in crate::drivers::keyboard::ps2)
fn enable_sending_scancodes() -> Result<(), u8> {
    wait_input_buffer_empty();
    write_input_buffer(DATA_PORT, command::ENABLE_SCANCODES);
    wait_output_buffer_full();
    match read_output_buffer() {
        response::ACK => Ok(()),
        resp => Err(resp),
    }
}

/// Disable Keyboard
///
/// Resets keyboard to power-on condition by clearing the output buffer,
/// resetting typematic options, resetting last typematic key and setting
/// default key types.
/// The keyboard then waits for the next instruction
pub(in crate::drivers::keyboard::ps2)
fn disable_sending_scancodes() -> Result<(), u8> {
    wait_input_buffer_empty();
    write_input_buffer(DATA_PORT, command::DISABLE_SCANCODES);
    wait_output_buffer_full();
    match read_output_buffer() {
        response::ACK => Ok(()),
        resp => Err(resp),
    }
}

/// Set Default
///
/// Resets keyboard to power-on condition by clearing the output buffer,
/// resetting typematic options, resetting last typematic key and setting
/// default key types.
/// The keyboard then continues scanning if scannig was enabled.
pub(in crate::drivers::keyboard::ps2)
fn restore_default_parameters() -> Result<(), u8> {
    wait_input_buffer_empty();
    write_input_buffer(DATA_PORT, command::RESET_DEFAULTS);
    wait_output_buffer_full();
    match read_output_buffer() {
        response::ACK => Ok(()),
        resp => Err(resp),
    }
}

/// Set All Keys to Typematic
///
/// PS/2 keyboard responds by clearing its output buffer and setting the key
/// type to _Typematic_.
///
/// ## Note
///
/// This command may be sent while using any Scan Code Set but only has effect
/// when Scan Code Set 3 is in use.
pub(in crate::drivers::keyboard::ps2)
fn set_all_keys_typematic() -> Result<(), u8> {
    wait_input_buffer_empty();
    write_input_buffer(DATA_PORT, command::SET_ALL_T);
    wait_output_buffer_full();
    match read_output_buffer() {
        response::ACK => Ok(()),
        resp => Err(resp),
    }
}

/// Set All Keys to Make/Break
///
/// PS/2 keyboard responds by clearing its output buffer and setting the key
/// type to _Make/Break_.
///
/// ## Note
///
/// This command may be sent while using any Scan Code Set but only has effect
/// when Scan Code Set 3 is in use.
pub(in crate::drivers::keyboard::ps2)
fn set_all_keys_make_break() -> Result<(), u8> {
    wait_input_buffer_empty();
    write_input_buffer(DATA_PORT, command::SET_ALL_MB);
    wait_output_buffer_full();
    match read_output_buffer() {
        response::ACK => Ok(()),
        resp => Err(resp),
    }
}

/// Set All Keys to Make
///
/// PS/2 keyboard responds by clearing its output buffer and setting the key
/// type to _Make_.
///
/// ## Note
///
/// This command may be sent while using any Scan Code Set but only has effect
/// when Scan Code Set 3 is in use.
pub(in crate::drivers::keyboard::ps2)
fn set_all_keys_make() -> Result<(), u8> {
    wait_input_buffer_empty();
    write_input_buffer(DATA_PORT, command::SET_ALL_M);
    wait_output_buffer_full();
    match read_output_buffer() {
        response::ACK => Ok(()),
        resp => Err(resp),
    }
}

/// Set All Keys to Typematic Make/Break
///
/// PS/2 keyboard responds by clearing its output buffer and setting the key
/// type to _Typematic Make/Break_.
///
/// ## Note
///
/// This command may be sent while using any Scan Code Set but only has effect
/// when Scan Code Set 3 is in use.
pub(in crate::drivers::keyboard::ps2)
fn set_all_keys_typematic_make_break() -> Result<(), u8> {
    wait_input_buffer_empty();
    write_input_buffer(DATA_PORT, command::SET_ALL_TMB);
    wait_output_buffer_full();
    match read_output_buffer() {
        response::ACK => Ok(()),
        resp => Err(resp),
    }
}

/// Set Key Type to Typematic
///
/// PS/2 keyboard responds by clearing its output buffer and then waiting for
/// the key ID (make code from Scan Code Set 3). The specified key type is then
/// set to _Typematic_.
///
/// ## Note
///
/// This command may be sent while using any Scan Code Set but only has effect
/// when Scan Code Set 3 is in use.
pub(in crate::drivers::keyboard::ps2)
fn set_key_typematic(key:u8) -> Result<(), u8> {
    wait_input_buffer_empty();
    write_input_buffer(DATA_PORT, command::SET_KEY_T);
    wait_output_buffer_full();
    match read_output_buffer() {
        response::ACK => (),
        resp => return Err(resp),
    }
    wait_input_buffer_empty();
    write_input_buffer(DATA_PORT, key);
    Ok(())
}

/// Set Key Type to Make/Break
///
/// PS/2 keyboard responds by clearing its output buffer and then waiting for
/// the key ID (make code from Scan Code Set 3). The specified key type is then
/// set to _Make/Break_.
///
/// ## Note
///
/// This command may be sent while using any Scan Code Set but only has effect
/// when Scan Code Set 3 is in use.
pub(in crate::drivers::keyboard::ps2)
fn set_key_make_break(key:u8) -> Result<(), u8> {
    wait_input_buffer_empty();
    write_input_buffer(DATA_PORT, command::SET_KEY_MB);
    wait_output_buffer_full();
    match read_output_buffer() {
        response::ACK => (),
        resp => return Err(resp),
    }
    wait_input_buffer_empty();
    write_input_buffer(DATA_PORT, key);
    Ok(())
}

/// Set Key Type to Make
///
/// PS/2 keyboard responds by clearing its output buffer and then waiting for
/// the key ID (make code from Scan Code Set 3). The specified key type is then
/// set to _Make_.
///
/// ## Note
///
/// This command may be sent while using any Scan Code Set but only has effect
/// when Scan Code Set 3 is in use.
pub(in crate::drivers::keyboard::ps2)
fn set_key_make(key:u8) -> Result<(), u8> {
    wait_input_buffer_empty();
    write_input_buffer(DATA_PORT, command::SET_KEY_M);
    wait_output_buffer_full();
    match read_output_buffer() {
        response::ACK => (),
        resp => return Err(resp),
    }
    wait_input_buffer_empty();
    write_input_buffer(DATA_PORT, key);
    Ok(())
}

/// Resend
///
/// Should be sent when a transmission error is detected from the keyboard
pub(in crate::drivers::keyboard::ps2)
fn resend_last_byte() -> Result<(), u8> {
    wait_input_buffer_empty();
    write_input_buffer(DATA_PORT, command::RESEND);
    wait_output_buffer_full();
    match read_output_buffer() {
        response::ACK => Ok(()),
        resp => Err(resp),
    }
}

/// Reset And Self-Test
///
/// Keyboard begins a program reset and Basic Assurance Test (BAT).
/// Keyboard returns a one byte completion code then sets default Scan Code Set.
pub(in crate::drivers::keyboard::ps2)
fn reset_and_test() -> Result<u8, u8> {
    wait_input_buffer_empty();
    write_input_buffer(DATA_PORT, command::RESET_AND_TEST);
    wait_output_buffer_full();
    match read_output_buffer() {
        response::ACK => (),
        resp => return Err(resp),
    }
    wait_output_buffer_full();
    let test_response = read_output_buffer();
    match test_response {
        response::TEST_OK|response::TEST_ERR1|response::TEST_ERR2 => Ok(test_response),
        _ => Err(test_response),
    }
}
