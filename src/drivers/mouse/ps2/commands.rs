use crate::drivers::chips::ps2::{
    wait_input_buffer_empty, wait_output_buffer_full, read_output_buffer
};
use crate::drivers::chips::ps2::commands as controller;


/// Mouse interface commands
///
/// These commands shall be written to port 0x60
mod command {
    /// Change scaling to 1:1
    pub(super) const
    SET_SCALING_1_1     : u8 = 0xE6;
    /// Change scaling to 2:1
    pub(super) const
    SET_SCALING_2_1     : u8 = 0xE7;
    /// Change resolution
    pub(super) const
    WRITE_RESOLUTION    : u8 = 0xE8;
    /// Request the status packet
    pub(super) const
    STATUS_REQUEST      : u8 = 0xE9;
    /// Enter stream mode
    pub(super) const
    SET_STREAM_MODE     : u8 = 0xEA;
    /// Request a data packet
    pub(super) const
    REQUEST_DATA_PACKET : u8 = 0xEB;
    /// Leave wrap mode
    pub(super) const
    DISABLE_WRAP_MODE   : u8 = 0xEC;
    /// Enter wrap mode
    pub(super) const
    ENABLE_WRAP_MODE    : u8 = 0xEE;
    /// Enter remote mode
    pub(super) const
    SET_REMOTE_MODE     : u8 = 0xF0;
    /// Query the device ID
    pub(super) const
    DEVICE_ID           : u8 = 0xF2;
    /// Change sample rate
    pub(super) const
    WRITE_SAMPLE_RATE   : u8 = 0xF3;
    /// Enable data reporting
    pub(super) const
    ENABLE_REPORTING    : u8 = 0xF4;
    /// Disable data reporting
    pub(super) const
    DISABLE_REPORTING   : u8 = 0xF5;
    /// Reset to default parameters
    pub(super) const
    RESET_DEFAULTS      : u8 = 0xF6;
    /// Resend the last byte
    pub(super) const
    RESEND              : u8 = 0xFE;
    /// Reset and perform self-testing
    pub(super) const
    RESET_AND_TEST      : u8 = 0xFF;
}


pub(in crate::drivers::mouse::ps2)
mod response {
    /// Self test passed
    pub(in crate::drivers::mouse::ps2) const
    TEST_OK             : u8 = 0xAA;
    /// Command acknowledged
    pub(in crate::drivers::mouse::ps2) const
    ACK                 : u8 = 0xFA;
    /// Detection error or internal buffer overrun
    pub(in crate::drivers::mouse::ps2) const
    ERROR               : u8 = 0xFC;
    /// Resend last command
    pub(in crate::drivers::mouse::ps2) const
    RESEND              : u8 = 0xFE;
}


/// Activate 1:1 Scaling
///
/// Sets the scaling of movement to 1:1, in which the reported movements
/// equals the movement counters.
pub(in crate::drivers::mouse::ps2)
fn write_scaling_1_1() -> Result<(), u8> {
    controller::write_second_device_input_buffer(command::SET_SCALING_1_1);
    wait_output_buffer_full();
    match read_output_buffer() {
        response::ACK => Ok(()),
        resp => Err(resp),
    }
}


/// Activate 2:1 Scaling
///
/// Sets the scaling of movement to 2:1, in which the reported movements are
/// scaled following an algorithm.
///
/// ## Scaling Algorithm
///
/// | Movement Counter | Reported Movement |
/// | :--------------: | :---------------: |
/// |         0        |         0         |
/// |         1        |         1         |
/// |         2        |         1         |
/// |         3        |         3         |
/// |         4        |         6         |
/// |         5        |         9         |
/// |       N > 5      |       2 * N       |
pub(in crate::drivers::mouse::ps2)
fn write_scaling_2_1() -> Result<(), u8> {
    controller::write_second_device_input_buffer(command::SET_SCALING_2_1);
    wait_output_buffer_full();
    match read_output_buffer() {
        response::ACK => Ok(()),
        resp => Err(resp),
    }
}


/// Set Count Resolution
///
/// Instructs the mouse about which resolution to use.
///
/// ## Format
///
/// - `0x00`: 1 count/mm
/// - `0x01`: 2 count/mm
/// - `0x02`: 4 count/mm
/// - `0x03`: 8 count/mm
///
/// ## Default
///
/// Defaults to 4 count/mm
pub(in crate::drivers::mouse::ps2)
fn write_resolution(byte:u8) -> Result<(), u8> {
    controller::write_second_device_input_buffer(command::WRITE_RESOLUTION);
    wait_output_buffer_full();
    match read_output_buffer() {
        response::ACK => (),
        resp => return Err(resp),
    }
    wait_input_buffer_empty();
    controller::write_second_device_input_buffer(byte);
    wait_output_buffer_full();
    match read_output_buffer() {
        response::ACK => Ok(()),
        resp => Err(resp),
    }
}


/// Status Request
///
/// The mouse responds with a 3 bytes status packet and then resets its
/// movement counters.
/// The first byte contains data about the currently pressed bottons and
/// some of the configuration options, the second byte represents the
/// resolution and the third byte represents the sample rate.
///
/// ## Format
///
/// _Format of the first byte_
///
/// | Bit | Description                         |
/// | :-: | :---------------------------------- |
/// |  0  | - **1** = Right button pressed      |
/// |     | - **0** = Right button not pressed  |
/// |  1  | - **1** = Middle button pressed     |
/// |     | - **0** = Middle button not pressed |
/// |  2  | - **1** = Left button pressed       |
/// |     | - **0** = Left button not pressed   |
/// |  3  | _reserved (must be 0)_              |
/// |  4  | - **1** = Scaling 2:1               |
/// |     | - **0** = Scaling 1:1               |
/// |  5  | - **1** = Data reporting enabled    |
/// |     | - **0** = Data reporting disabled   |
/// |  6  | - **1** = Remote mode enabled       |
/// |     | - **0** = Stream mode enabled       |
/// |  7  | _reserved (must be 0)_              |
pub(in crate::drivers::mouse::ps2)
fn status_request() -> Result<[u8;3], u8> {
    controller::write_second_device_input_buffer(command::STATUS_REQUEST);
    wait_output_buffer_full();
    match read_output_buffer() {
        response::ACK => (),
        resp => return Err(resp),
    }
    let mut bytes = [0x00_u8; 3];
    wait_output_buffer_full();
    bytes[0] = read_output_buffer();
    wait_output_buffer_full();
    bytes[1] = read_output_buffer();
    wait_output_buffer_full();
    bytes[2] = read_output_buffer();
    Ok(bytes)
}


/// Enter Stream Mode
///
/// The mouse resets its movement counters and then enters stream mode,
/// in which data packets are automatically sent to the controller whenever
/// a movement or a state change is detected.
pub(in crate::drivers::mouse::ps2)
fn enter_stream_mode() -> Result<(), u8> {
    controller::write_second_device_input_buffer(command::SET_STREAM_MODE);
    wait_output_buffer_full();
    match read_output_buffer() {
        response::ACK => Ok(()),
        resp => Err(resp),
    }
}


/// Request Data Packet
///
/// Requests a data packet to the device, which then resets its movement
/// counters.
/// This is the only way to read data when Remote Mode is active.
pub(in crate::drivers::mouse::ps2)
fn request_data_packet() -> Result<(), u8> {
    controller::write_second_device_input_buffer(command::REQUEST_DATA_PACKET);
    wait_output_buffer_full();
    match read_output_buffer() {
        response::ACK => Ok(()),
        resp => Err(resp),
    }
}


/// Disable Wrap Mode
///
/// The mouse exits from Wrap Mode and returns to the last mode that was active
/// prior to entering Wrap Mode (Stream Mode or Remote Mode), then resets its
/// movement counters.
pub(in crate::drivers::mouse::ps2)
fn disable_wrap_mode() -> Result<(), u8> {
    controller::write_second_device_input_buffer(command::DISABLE_WRAP_MODE);
    wait_output_buffer_full();
    match read_output_buffer() {
        response::ACK => Ok(()),
        resp => Err(resp),
    }
}


/// Enable Wrap Mode
///
/// The mouse resets its movement counters and then enters in Wrap Mode, in
/// which every command sent to the device is echoed back to the host, exception
/// made for the "Reset" and "Disable Wrap Mode" commands.
pub(in crate::drivers::mouse::ps2)
fn enable_wrap_mode() -> Result<(), u8> {
    controller::write_second_device_input_buffer(command::ENABLE_WRAP_MODE);
    wait_output_buffer_full();
    match read_output_buffer() {
        response::ACK => Ok(()),
        resp => Err(resp),
    }
}


/// Enter Remote Mode
///
/// Upon receiving the command the mouse sends a single data packet, then
/// resets its movement counters and enters remote mode, in which the device
/// updates its internal counters and state flags but does not automatically
/// send data packets to the controller.
pub(in crate::drivers::mouse::ps2)
fn enter_remote_mode() -> Result<(), u8> {
    controller::write_second_device_input_buffer(command::SET_REMOTE_MODE);
    wait_output_buffer_full();
    match read_output_buffer() {
        response::ACK => Ok(()),
        resp => Err(resp),
    }
}


/// Read Mouse ID
///
/// PS/2 mice respond with a one byte mouse ID of `0x00`, `0x03` or `0x04`.
/// The mouse should also reset its movement counters.
pub(in crate::drivers::mouse::ps2)
fn read_device_id() -> Result<u8, u8> {
    controller::write_second_device_input_buffer(command::DEVICE_ID);
    wait_output_buffer_full();
    match read_output_buffer() {
        response::ACK => (),
        resp => return Err(resp),
    }
    wait_output_buffer_full();
    Ok(read_output_buffer())
}


/// Set Sample Rate
///
/// Instructs the device to use the given sample rate.
/// This also resets the movement counters.
///
/// ## Format
///
/// - `0x0A` = 10 samples/sec
/// - `0x14` = 20 samples/sec
/// - `0x28` = 40 samples/sec
/// - `0x3C` = 60 samples/sec
/// - `0x50` = 80 samples/sec
/// - `0x64` = 100 samples/sec
/// - `0xC8` = 200 samples/sec
///
/// ## Default
///
/// Defaults to 100 samples/sec
pub(in crate::drivers::mouse::ps2)
fn write_sample_rate(byte:u8) -> Result<(), u8> {
    controller::write_second_device_input_buffer(command::WRITE_SAMPLE_RATE);
    wait_output_buffer_full();
    match read_output_buffer() {
        response::ACK => (),
        resp => return Err(resp),
    }
    wait_input_buffer_empty();
    controller::write_second_device_input_buffer(byte);
    wait_output_buffer_full();
    match read_output_buffer() {
        response::ACK => Ok(()),
        resp => Err(resp),
    }
}


/// Enable Data Reporting
///
/// The mouse resets its movement counters and then enables data reporting.
/// Only affects Stream Mode.
///
/// ## Note
///
/// In Stream Mode, data reporting should be disabled before issuing any of
/// the commands.
pub(in crate::drivers::mouse::ps2)
fn enable_data_reporting() -> Result<(), u8> {
    controller::write_second_device_input_buffer(command::ENABLE_REPORTING);
    wait_output_buffer_full();
    match read_output_buffer() {
        response::ACK => Ok(()),
        resp => Err(resp),
    }
}


/// Disable Data Reporting
///
/// The mouse resets its movement counters and then disables data reporting.
/// Only affects Stream Mode.
///
/// ## Note
///
/// In Stream Mode, data reporting should be disabled before issuing any of
/// the commands.
pub(in crate::drivers::mouse::ps2)
fn disable_data_reporting() -> Result<(), u8> {
    controller::write_second_device_input_buffer(command::DISABLE_REPORTING);
    wait_output_buffer_full();
    match read_output_buffer() {
        response::ACK => Ok(()),
        resp => Err(resp),
    }
}


/// Set Default
///
/// Resets the mouse to power-on condition by clearing the movement counters,
/// entering Stream Mode and resetting the scaling, sample rate, resolution and
/// data reporting options.
pub(in crate::drivers::mouse::ps2)
fn restore_default_parameters() -> Result<(), u8> {
    controller::write_second_device_input_buffer(command::RESET_DEFAULTS);
    wait_output_buffer_full();
    match read_output_buffer() {
        response::ACK => Ok(()),
        resp => Err(resp),
    }
}

/// Resend
///
/// Should be sent when a transmission error is detected from the mouse.
pub(in crate::drivers::mouse::ps2)
fn resend_last_byte() -> Result<(), u8> {
    controller::write_second_device_input_buffer(command::RESEND);
    wait_output_buffer_full();
    match read_output_buffer() {
        response::ACK => Ok(()),
        resp => Err(resp),
    }
}


/// Reset And Self-Test
///
/// The mouse begins a program reset and Basic Assurance Test (BAT).
/// The device returns a one byte completion code for the test and then
/// sends its Device ID.
pub(in crate::drivers::mouse::ps2)
fn reset_and_test() -> Result<(u8,u8), u8> {
    controller::write_second_device_input_buffer(command::RESET_AND_TEST);
    wait_output_buffer_full();
    match read_output_buffer() {
        response::ACK => (),
        resp => return Err(resp),
    }
    wait_output_buffer_full();
    let test_response = read_output_buffer();
    let response = match test_response {
        response::TEST_OK|response::ERROR => Ok(()),
        _ => Err(()),
    };
    wait_output_buffer_full();
    let device_id = read_output_buffer();
    match response {
        Ok(_) => Ok((test_response, device_id)),
        Err(_) => Err(test_response),
    }
}
