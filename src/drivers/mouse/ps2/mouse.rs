use crate::drivers::chips::pic;
use crate::drivers::chips::ps2 as controller;
use crate::drivers::mouse;
use crate::drivers::mouse::{
    MouseType, MouseEvent
};
use crate::drivers::mouse::ps2::commands;


const
BITS_BUTTONS_1_2_3  : u8 = 0b00000111;
const
BITS_BUTTONS_4_5    : u8 = 0b00110000;

const
BIT_X_SIGN          : u8 = 0b00010000;
const
BIT_Y_SIGN          : u8 = 0b00100000;

const
BITS_Z_SIGNED       : u8 = 0b00011111;
const
BITS_Z_UNSIGNED     : u8 = 0b00001111;

const
DEVICE_ID_DEFAULT       : u8 = 0x00;
const
DEVICE_ID_INTELLIMOUSE  : u8 = 0x03;
const
DEVICE_ID_EXTRA_BUTTONS : u8 = 0x04;

const
BUFFER_SIZE : usize = 64;


pub(crate)
struct MouseError (u8);

impl MouseError {
    pub(crate)
    fn resend_requested(&self) -> bool {
        self.0 == commands::response::RESEND
    }
}

impl From<u8> for MouseError {
    fn from(byte:u8) -> Self {
        Self(byte)
    }
}


#[repr(u8)]
#[allow(non_camel_case_types)]
#[derive(Clone,Copy)]
enum Scaling {
    S_1_1 = 0x00,
    S_2_1 = 0x01,
}

impl Default for Scaling {
    fn default() -> Self {
        Self::S_1_1
    }
}


#[repr(u8)]
#[allow(non_camel_case_types)]
#[derive(Clone,Copy)]
enum Resolution {
    /// 1 count/mm
    R_1 = 0x00,
    /// 2 count/mm
    R_2 = 0x01,
    /// 4 count/mm
    R_4 = 0x02,
    /// 8 count/mm
    R_8 = 0x03,
}

impl Default for Resolution {
    fn default() -> Self {
        Self::R_4
    }
}


#[repr(u8)]
#[allow(non_camel_case_types)]
#[derive(Clone,Copy)]
enum SampleRate {
    SR_10  = 10,
    SR_20  = 20,
    SR_40  = 40,
    SR_60  = 60,
    SR_80  = 80,
    SR_100 = 100,
    SR_200 = 200,
}

impl Default for SampleRate {
    fn default() -> Self {
        Self::SR_100
    }
}


#[derive(Clone,Copy,PartialEq)]
enum Mode {
    Stream,
    Remote,
    Wrap,
}

impl Default for Mode {
    fn default() -> Self {
        Self::Stream
    }
}


/// Temporarily stores the [`MouseEvent`]s queue
struct EventsBuffer {
    idx : usize,
    max : usize,
    buf : [MouseEvent; BUFFER_SIZE],
}

impl EventsBuffer {
    fn push(&mut self, event:MouseEvent) {
        self.buf[self.max] = event;
        self.max += 1;
        if self.max == BUFFER_SIZE {
            self.max = 0;
        }
    }

    fn pop(&mut self) -> Option<MouseEvent> {
        if self.idx == self.max {
            return None;
        }
        let event = self.buf[self.idx];
        self.idx += 1;
        if self.idx == BUFFER_SIZE {
            self.idx = 0;
        }
        Some(event)
    }
}

impl Default for EventsBuffer {
    fn default() -> Self {
        let event = MouseEvent {
            buttons    : 0x0000,
            x_movement : 0,
            y_movement : 0,
            z_movement : 0,
        };
        Self {
            idx : 0,
            max : 0,
            buf : [event; BUFFER_SIZE],
        }
    }
}


/// Temporarily stores the bytes needed to generate a [`MouseEvent`]
struct ScanBuffer {
    size : usize,
    buf  : [u8; 4],
}

impl ScanBuffer {
    fn scan(&mut self, byte:u8, intellimouse:bool, extra_buttons:bool) -> Option<MouseEvent> {
        self.buf[self.size] = byte;
        self.size += 1;
        match (intellimouse, self.size) {
            (false, 3) => {
                let event = Some(self.event_3_bytes());
                *self = Self::default();
                event
            },
            (true, 4) => {
                let event = Some(self.event_4_bytes(extra_buttons));
                *self = Self::default();
                event
            },
            _ => None,
        }
    }

    fn event_3_bytes(&mut self) -> MouseEvent {
        let state = self.buf[0];
        let x_absolute = self.buf[1] as i16;
        let x_sign = (state & BIT_X_SIGN) as i16;
        let x_relative = x_absolute - (x_sign << 4);
        let y_absolute = self.buf[2] as i16;
        let y_sign = (state & BIT_Y_SIGN) as i16;
        let y_relative = y_absolute - (y_sign << 3);
        let buttons_1_2_3 = (state & BITS_BUTTONS_1_2_3) as u16;
        MouseEvent {
            buttons    : buttons_1_2_3,
            x_movement : x_relative,
            y_movement : y_relative,
            z_movement : 0,
        }
    }

    fn event_4_bytes(&mut self, extra_buttons:bool) -> MouseEvent {
        let mut event = self.event_3_bytes();
        let state = self.buf[3];
        if extra_buttons {
            event.buttons |= (state & BITS_BUTTONS_4_5) as u16;
            event.z_movement = (state & BITS_Z_UNSIGNED) as i16;
        } else {
            event.z_movement = (state & BITS_Z_SIGNED) as i16;
        }
        event
    }
}

impl Default for ScanBuffer {
    fn default() -> Self {
        Self {
            size : 0,
            buf  : [0; 4],
        }
    }
}


/// Represents a `PS/2` mouse
///
/// ## Default
///
/// At power-on or software reset the keyboard loads the following default values:
/// - Sample rate: 100 samples/sec
/// - Resolution: 4 counts/mm
/// - Scaling: 1:1
/// - Data reporting: disabled
/// - Mode: stream
#[derive(Default)]
pub(crate)
struct Mouse {
    device_id     : u8,
    scaling       : Scaling,
    resolution    : Resolution,
    sample_rate   : SampleRate,
    intellimouse  : bool,
    extra_buttons : bool,
    mode          : Mode,
    scan_buffer   : ScanBuffer,
    events_buffer : EventsBuffer,
}

impl Mouse {
    /// Returns the current device ID
    pub(crate)
    fn device_id(&self) -> u8 {
        self.device_id
    }

    /// Reads the device ID from the mouse
    fn retrieve_device_id(&mut self) -> Result<(), MouseError> {
        pic::mask_irq(12);
        self.device_id = commands::read_device_id()?;
        pic::unmask_irq(12);
        Ok(())
    }

    /// Writes the given scaling to the mouse
    fn apply_scaling(&mut self, scaling:Scaling) -> Result<(), MouseError> {
        pic::mask_irq(12);
        self.scaling = scaling;
        match scaling {
            Scaling::S_1_1 => commands::write_scaling_1_1()?,
            Scaling::S_2_1 => commands::write_scaling_2_1()?,
        }
        pic::unmask_irq(12);
        Ok(())
    }

    /// Writes the given resolution to the mouse
    fn apply_resolution(&mut self, resolution:Resolution) -> Result<(), MouseError> {
        pic::mask_irq(12);
        self.resolution = resolution;
        commands::write_resolution(resolution as u8)?;
        pic::unmask_irq(12);
        Ok(())
    }

    /// Writes the given sample rate to the mouse
    fn apply_sample_rate(&mut self, sample_rate:SampleRate) -> Result<(), MouseError> {
        pic::mask_irq(12);
        self.sample_rate = sample_rate;
        commands::write_sample_rate(sample_rate as u8)?;
        pic::unmask_irq(12);
        Ok(())
    }

    /// Activates the stream mode for the mouse
    fn enter_stream_mode(&mut self) -> Result<(), MouseError> {
        if self.mode == Mode::Stream {
            return Ok(());
        }
        pic::mask_irq(12);
        match self.mode {
            Mode::Wrap => commands::disable_wrap_mode()?,
            _ => (),
        }
        self.mode = Mode::Stream;
        commands::enter_stream_mode()?;
        pic::unmask_irq(12);
        Ok(())
    }

    /// Activates the remote mode for the mouse
    fn enter_remote_mode(&mut self) -> Result<(), MouseError> {
        if self.mode == Mode::Remote {
            return Ok(());
        }
        pic::mask_irq(12);
        match self.mode {
            Mode::Wrap => commands::disable_wrap_mode()?,
            _ => (),
        }
        self.mode = Mode::Remote;
        commands::enter_remote_mode()?;
        pic::unmask_irq(12);
        Ok(())
    }

    /// Activates the wrap mode for the mouse
    fn enter_wrap_mode(&mut self) -> Result<(), MouseError> {
        if self.mode == Mode::Wrap {
            return Ok(());
        }
        pic::mask_irq(12);
        self.mode = Mode::Wrap;
        commands::enable_wrap_mode()?;
        pic::unmask_irq(12);
        Ok(())
    }

    /// Attempts to enable the scroll wheel for the mouse
    fn try_enable_intellimouse(&mut self) -> Result<bool, MouseError> {
        pic::mask_irq(12);
        let result = enable_intellimouse(self);
        pic::unmask_irq(12);
        result
    }

    /// Attempts to enable the forth and fifth buttons for the mouse
    fn try_enable_extra_buttons(&mut self) -> Result<bool, MouseError> {
        if !self.intellimouse {
            return Ok(false);
        }
        pic::mask_irq(12);
        let result = enable_extra_buttons(self);
        pic::unmask_irq(12);
        result
    }

    /// Reads the controller output buffer
    ///
    /// ## Warning
    ///
    /// This function assumes that the data in the output buffer of the PS/2
    /// controller actually contains data sent by the mouse.
    fn read_buffer(&mut self) {
        let scancode = controller::read_output_buffer();
        if let Some(key) = self.scan_buffer.scan(scancode, self.intellimouse, self.extra_buttons) {
            self.events_buffer.push(key);
        }
    }
}

impl mouse::Mouse for Mouse {
    fn mouse_type(&self) -> MouseType {
        MouseType::PS2
    }

    fn handle_interrupt(&mut self) {
        self.read_buffer();
    }

    fn fetch(&mut self) {
        if controller::output_buffer_full() {
            self.read_buffer();
        }
    }

    fn next(&mut self) -> Option<MouseEvent> {
        self.events_buffer.pop()
    }
}


fn enable_intellimouse(mouse:&mut Mouse) -> Result<bool, MouseError> {
    commands::write_sample_rate(SampleRate::SR_200 as u8)?;
    commands::write_sample_rate(SampleRate::SR_100 as u8)?;
    commands::write_sample_rate(SampleRate::SR_80 as u8)?;
    mouse.device_id = commands::read_device_id()?;
    match mouse.device_id {
        DEVICE_ID_DEFAULT => {
            mouse.intellimouse &= false;
            return Ok(false);
        },
        DEVICE_ID_INTELLIMOUSE|DEVICE_ID_EXTRA_BUTTONS => {
            mouse.intellimouse |= true;
            return Ok(true);
        },
        _ => return Err(mouse.device_id.into()), // unexpected ID
    }
}

fn enable_extra_buttons(mouse:&mut Mouse) -> Result<bool, MouseError> {
    commands::write_sample_rate(SampleRate::SR_200 as u8)?;
    commands::write_sample_rate(SampleRate::SR_200 as u8)?;
    commands::write_sample_rate(SampleRate::SR_80 as u8)?;
    mouse.device_id = commands::read_device_id()?;
    match mouse.device_id {
        DEVICE_ID_DEFAULT|DEVICE_ID_INTELLIMOUSE => {
            mouse.extra_buttons &= false;
            return Ok(false);
        },
        DEVICE_ID_EXTRA_BUTTONS => {
            mouse.extra_buttons |= true;
            return Ok(true);
        },
        _ => return Err(mouse.device_id.into()), // unexpected ID
    }
}


pub(in crate::drivers)
fn init(mouse:&mut Mouse) -> Result<(), MouseError> {
    controller::clear_output_buffer();
    commands::disable_data_reporting();

    controller::clear_output_buffer();
    let (test_response, _) = commands::reset_and_test()?;
    match test_response {
        commands::response::TEST_OK => (),
        commands::response::ERROR => return Err(test_response.into()),
        byte => return Err(byte.into()),
    }
    // TODO:
    //   Log if test failed with error

    if enable_intellimouse(mouse)? {
        let _ = enable_extra_buttons(mouse)?;
    }
    commands::write_sample_rate(SampleRate::SR_100 as u8)?;

    commands::enable_data_reporting()?;

    Ok(())
}
