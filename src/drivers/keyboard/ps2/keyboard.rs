use crate::drivers::chips::pic;
use crate::drivers::chips::ps2 as controller;
use crate::drivers::keyboard;
use crate::drivers::keyboard::{
    KeyboardType, KeyEvent, KeyState, Key
};
use crate::drivers::keyboard::ps2::commands;
use crate::drivers::keyboard::ps2::scancodes;
use crate::drivers::keyboard::ps2::scancodes::ScancodeSet;


const
BUFFER_SIZE : usize = 64;


pub(crate)
struct KeyboardError (u8);

impl KeyboardError {
    pub(crate)
    fn resend_requested(&self) -> bool {
        self.0 == commands::response::RESEND
    }
}

impl From<u8> for KeyboardError {
    fn from(byte:u8) -> Self {
        Self(byte)
    }
}


#[derive(Default)]
struct LedsState {
    byte : u8,
}

impl LedsState {
    fn as_byte(&self) -> u8 {
        self.byte
    }

    fn toggle_scroll_lock(&mut self) {
        self.byte ^= 0b00000001;
    }

    fn toggle_num_lock(&mut self) {
        self.byte ^= 0b00000010;
    }

    fn toggle_caps_lock(&mut self) {
        self.byte ^= 0b00000100;
    }
}


#[repr(u8)]
#[allow(non_camel_case_types)]
#[derive(Clone,Copy)]
enum TypematicRepeatRate {
    HZ_30_0 = 0b00000000,
    HZ_26_7 = 0b00000001,
    HZ_24_0 = 0b00000010,
    HZ_21_8 = 0b00000011,
    HZ_20_7 = 0b00000100,
    HZ_18_5 = 0b00000101,
    HZ_17_1 = 0b00000110,
    HZ_16_0 = 0b00000111,
    HZ_15_0 = 0b00001000,
    HZ_13_3 = 0b00001001,
    HZ_12_0 = 0b00001010,
    HZ_10_9 = 0b00001011,
    HZ_10_0 = 0b00001100,
    HZ_9_2  = 0b00001101,
    HZ_8_6  = 0b00001110,
    HZ_8_0  = 0b00001111,
    HZ_7_5  = 0b00010000,
    HZ_6_7  = 0b00010001,
    HZ_6_0  = 0b00010010,
    HZ_5_5  = 0b00010011,
    HZ_5_0  = 0b00010100,
    HZ_4_6  = 0b00010101,
    HZ_4_3  = 0b00010110,
    HZ_4_0  = 0b00010111,
    HZ_3_7  = 0b00011000,
    HZ_3_3  = 0b00011001,
    HZ_3_0  = 0b00011010,
    HZ_2_7  = 0b00011011,
    HZ_2_5  = 0b00011100,
    HZ_2_3  = 0b00011101,
    HZ_2_1  = 0b00011110,
    HZ_2_0  = 0b00011111,
}


#[repr(u8)]
#[allow(non_camel_case_types)]
#[derive(Clone,Copy)]
enum TypematicRepeatDelay {
    MS_250  = 0b00000000,
    MS_500  = 0b00100000,
    MS_750  = 0b01000000,
    MS_1000 = 0b01100000,
}


struct TypematicOptions {
    repeat_rate  : TypematicRepeatRate,
    repeat_delay : TypematicRepeatDelay,
}

impl TypematicOptions {
    fn as_byte(&self) -> u8 {
        self.repeat_rate as u8 | self.repeat_delay as u8
    }

    fn repeat_rate(&self) -> TypematicRepeatRate {
        self.repeat_rate
    }

    fn set_repeat_rate(&mut self, rate:TypematicRepeatRate) {
        self.repeat_rate = rate;
    }

    fn repeat_delay(&self) -> TypematicRepeatDelay {
        self.repeat_delay
    }

    fn set_repeat_delay(&mut self, delay:TypematicRepeatDelay) {
        self.repeat_delay = delay;
    }
}

impl Default for TypematicOptions {
    fn default() -> Self {
        Self {
            repeat_rate  : TypematicRepeatRate::HZ_10_9,
            repeat_delay : TypematicRepeatDelay::MS_500,
        }
    }
}


/// Temporarily stores the [`KeyEvent`]s queue
struct EventsBuffer {
    idx : usize,
    max : usize,
    buf : [KeyEvent; BUFFER_SIZE],
}

impl EventsBuffer {
    fn push(&mut self, event:KeyEvent) {
        self.buf[self.max] = event;
        self.max += 1;
        if self.max == BUFFER_SIZE {
            self.max = 0;
        }
    }

    fn pop(&mut self) -> Option<KeyEvent> {
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
        let event = KeyEvent {
            key   : Key::Escape,
            state : KeyState::Released,
        };
        Self {
            idx : 0,
            max : 0,
            buf : [event; BUFFER_SIZE],
        }
    }
}


/// Temporarily stores the scancodes needed to generate a [`KeyEvent`]
struct ScanBuffer {
    size : usize,
    buf  : [u8; 8],
}

impl ScanBuffer {
    fn scan(&mut self, code:u8, set:ScancodeSet) -> Option<KeyEvent> {
        self.buf[self.size] = code;
        self.size += 1;
        let event = match set {
            ScancodeSet::Set1 => scancodes::set_1::key_from(&self.buf[..self.size]),
            ScancodeSet::Set2 => scancodes::set_2::key_from(&self.buf[..self.size]),
            ScancodeSet::Set3 => scancodes::set_3::key_from(&self.buf[..self.size]),
        };
        match event {
            Err(()) => {
                *self = ScanBuffer::default();
                None
            },
            Ok(k) => match k {
                Some(_) => {
                    *self = ScanBuffer::default();
                    k
                },
                None => k,
            },
        }
    }
}

impl Default for ScanBuffer {
    fn default() -> Self {
        Self {
            size : 0,
            buf  : [0; 8],
        }
    }
}


/// Represents a PS/2 keyboard
///
/// ## Default
///
/// At power-on or software reset the keyboard loads the following default values:
/// - Typematic delay: 500 ms
/// - Typematic rate: 10.9 hz
/// - Scancode set: 2
/// - Set all keys: typematic/make/break
/// - Set all leds: off
#[derive(Default)]
pub(crate)
struct Keyboard {
    device_id          : [u8; 2],
    leds_state         : LedsState,
    typematic_options  : TypematicOptions,
    scancode_set       : ScancodeSet,
    scan_buffer        : ScanBuffer,
    events_buffer      : EventsBuffer,
}

impl Keyboard {
    /// Echoes the device
    pub(crate)
    fn echo(&mut self) -> Result<(), KeyboardError> {
        pic::mask_irq(1);
        commands::send_echo()?;
        pic::unmask_irq(1);
        Ok(())
    }

    /// Toggles the state of the _ScrollLock_ led on the keyboard
    pub(crate)
    fn toggle_scroll_lock_led(&mut self) -> Result<(), KeyboardError> {
        pic::mask_irq(1);
        self.leds_state.toggle_scroll_lock();
        commands::write_leds_state(self.leds_state.as_byte())?;
        pic::unmask_irq(1);
        Ok(())
    }

    /// Toggles the state of the _NumLock_ led on the keyboard
    pub(crate)
    fn toggle_num_lock_led(&mut self) -> Result<(), KeyboardError> {
        pic::mask_irq(1);
        self.leds_state.toggle_num_lock();
        commands::write_leds_state(self.leds_state.as_byte())?;
        pic::unmask_irq(1);
        Ok(())
    }

    /// Toggles the state of the _CapsLock_ led on the keyboard
    pub(crate)
    fn toggle_caps_lock_led(&mut self) -> Result<(), KeyboardError> {
        pic::mask_irq(1);
        self.leds_state.toggle_caps_lock();
        commands::write_leds_state(self.leds_state.as_byte())?;
        pic::unmask_irq(1);
        Ok(())
    }

    /// Returns the current scancodes set
    fn scancode_set(&self) -> ScancodeSet {
        self.scancode_set
    }

    /// Reads the scancodes set from the keyboard
    fn retrieve_scancode_set(&mut self) -> Result<(), KeyboardError> {
        pic::mask_irq(1);
        self.scancode_set = commands::read_scancode_set()?.try_into()?;
        pic::unmask_irq(1);
        Ok(())
    }

    /// Writes the given scancodes set to the keyboard
    fn apply_scancode_set(&mut self, set:ScancodeSet) -> Result<(), KeyboardError> {
        pic::mask_irq(1);
        self.scancode_set = set;
        commands::write_scancode_set(set.into())?;
        pic::unmask_irq(1);
        Ok(())
    }

    /// Returns the current device ID
    pub(crate)
    fn device_id(&self) -> [u8; 2] {
        self.device_id
    }

    /// Reads the device ID from the keyboard
    fn retrieve_device_id(&mut self) -> Result<(), KeyboardError> {
        pic::mask_irq(1);
        self.device_id = commands::read_device_id()?;
        pic::unmask_irq(1);
        Ok(())
    }

    /// Returns a mutable reference to the current typematic options
    pub(crate)
    fn typematic_options(&mut self) -> &mut TypematicOptions {
        &mut self.typematic_options
    }

    /// Enables or disables the communication of scancodes from the keyboard
    fn set_scancodes_communication(&self, state:bool) -> Result<(), KeyboardError> {
        pic::mask_irq(1);
        match state {
            true => commands::enable_sending_scancodes(),
            false => commands::disable_sending_scancodes(),
        }?;
        pic::unmask_irq(1);
        Ok(())
    }

    /// Reads the controller output buffer
    ///
    /// ## Warning
    ///
    /// This function assumes that the data in the output buffer of the PS/2
    /// controller actually contains data sent by the keyboard.
    fn read_buffer(&mut self) {
        let scancode = controller::read_output_buffer();
        if let Some(key) = self.scan_buffer.scan(scancode, self.scancode_set) {
            self.events_buffer.push(key);
        }
    }
}

impl keyboard::Keyboard for Keyboard {
    fn keyboard_type(&self) -> KeyboardType {
        KeyboardType::PS2
    }

    fn handle_interrupt(&mut self) {
        self.read_buffer();
    }

    fn fetch(&mut self) {
        if controller::output_buffer_full() {
            self.read_buffer();
        }
    }

    fn next(&mut self) -> Option<KeyEvent> {
        self.events_buffer.pop()
    }
}


pub(in crate::drivers)
fn init(keyboard:&mut Keyboard) -> Result<(), KeyboardError> {
    use controller::controller::controller as get_controller;

    controller::clear_output_buffer();
    commands::disable_sending_scancodes()?;

    controller::clear_output_buffer();
    let _ = commands::send_echo()?;

    controller::clear_output_buffer();
    let test_response = commands::reset_and_test()?;
    match test_response {
        commands::response::TEST_OK => (),
        commands::response::TEST_ERR1 => return Err(test_response.into()),
        commands::response::TEST_ERR2 => return Err(test_response.into()),
        byte => return Err(byte.into()),
    }
    // TODO:
    //   Log if test failed with error

    controller::clear_output_buffer();
    keyboard.device_id = commands::read_device_id()?;

    controller::clear_output_buffer();
    let cr = get_controller().command_register();
    keyboard.scancode_set = match cr.scancodes_translation() {
        true  => ScancodeSet::Set1,
        false => commands::read_scancode_set()?.try_into()?,
    };

    controller::clear_output_buffer();
    commands::enable_sending_scancodes()?;

    Ok(())
}
