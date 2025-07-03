use crate::drivers::chips::ps2::clear_output_buffer;
use crate::drivers::chips::ps2::commands;
use crate::drivers::chips::ps2::registers::*;
use crate::drivers::keyboard::ps2::keyboard::{self, Keyboard};
use crate::drivers::mouse::ps2::mouse::{self, Mouse};
use crate::memory::{Cast, Init};


static mut controller_ptr : u64 = 0;


#[repr(u8)]
#[derive(Clone,Copy)]
pub(crate)
enum CompatibilityMode {
    AT,
    PS2,
}

impl Default for CompatibilityMode {
    fn default() -> Self {
        Self::PS2
    }
}


#[derive(Default)]
pub(crate)
struct Controller {
    compatibility_mode : CompatibilityMode,
    keyboard           : Option<Keyboard>,
    mouse              : Option<Mouse>,
}

impl Controller {
    /// Returns the current [`CompatibilityMode`]
    pub(crate)
    fn compatibility_mode(&self) -> CompatibilityMode {
        self.compatibility_mode
    }

    /// Reads the value stored in the _command register_
    pub(crate)
    fn command_register(&self) -> CommandRegister {
        read_command_register().into()
    }

    /// Writes the given value to the _command register_
    pub(crate)
    fn apply_command_register(&self, cr:CommandRegister) {
        write_command_register(cr.into());
    }

    /// Reads the value stored in the _status register_
    pub(crate)
    fn status_flags(&self) -> StatusRegister {
        read_status_register().into()
    }

    /// Returns a reference to the PS/2 Keyboard instance, if any
    pub(crate)
    fn keyboard(&mut self) -> Option<&mut Keyboard> {
        self.keyboard.as_mut()
    }

    /// Returns a reference to the PS/2 Mouse instance, if any
    pub(crate)
    fn mouse(&mut self) -> Option<&mut Mouse> {
        self.mouse.as_mut()
    }

    pub(crate)
    fn perform_self_test(&self) -> Result<(), ()> {
        match commands::self_test() {
            commands::response::SELF_TEST_OK => Ok(()),
            _ => Err(()),
        }
    }
}

impl Init<u64> for Controller {
    fn init(ptr:u64) {
        unsafe {
            controller_ptr = ptr;
            *Self::cast_mut(ptr) = Self::default();
        }
    }
}

impl Cast<u64> for Controller {
    fn cast(ptr:u64) -> *const Self {
        ptr as *const Self
    }

    fn cast_mut(ptr:u64) -> *mut Self {
        ptr as *mut Self
    }
}


/// Returns a mutable reference to the [`Controller`] instance
///
/// ## Warning
///
/// This function doesn't check whether the [`Controller`] instance is
/// valid. Do not use before it has been initialized.
pub(crate)
fn controller() -> &'static mut Controller {
    unsafe { &mut *Controller::cast_mut(controller_ptr) }
}


/// Handles a keyboard interrupt event
///
/// The event is forwarded to the keyboard instance if any, or discarded
/// otherwise.
pub(crate)
fn handle_keyboard_interrupt() {
    use crate::drivers::keyboard::Keyboard;

    if let Some(keyboard) = controller().keyboard() {
        keyboard.handle_interrupt();
    }
}

/// Handles a mouse interrupt event
///
/// The event is forwarded to the mouse instance if any, or discarded
/// otherwise.
pub(crate)
fn handle_mouse_interrupt() {
    use crate::drivers::mouse::Mouse;

    if let Some(mouse) = controller().mouse() {
        mouse.handle_interrupt();
    }
}


pub(in crate::drivers::chips::ps2)
fn init() {
    // the controller does not start operating until the self test
    // command is sent and successfully completed
    clear_output_buffer();
    match commands::self_test() {
        commands::response::SELF_TEST_OK => (),
        _ => crate::panic("PS/2 controller test failed"),
    }

    let controller = controller();
    let mut cr = controller.command_register();

    // enable the I/O ports
    clear_output_buffer();
    commands::enable_first_io_port();
    cr.set_first_io_port_enabled(true);
    clear_output_buffer();
    commands::enable_second_io_port();
    cr.set_second_io_port_enabled(true);
    controller.apply_command_register(cr);
    clear_output_buffer();

    // define compatibility mode
    cr = controller.command_register();
    let ps2_mode : bool;
    (ps2_mode, controller.compatibility_mode) = match cr.second_io_port_enabled() {
        true  => (true, CompatibilityMode::PS2),
        false => (false, CompatibilityMode::AT),
    };
    // TODO:
    //   Log the compatibility mode

    clear_output_buffer();
    match commands::test_first_io_port() {
        commands::response::IO_PORT_TEST_NO_ERROR => controller.keyboard = Some(Keyboard::default()),
        commands::response::IO_PORT_TEST_CLOCK_LOW => crate::tty::print("[PS/2]> first I/O port clock low"),
        commands::response::IO_PORT_TEST_CLOCK_HIGH => crate::tty::print("[PS/2]> first I/O port clock high"),
        commands::response::IO_PORT_TEST_DATA_LOW => crate::tty::print("[PS/2]> first I/O port data low"),
        commands::response::IO_PORT_TEST_DATA_HIGH => crate::tty::print("[PS/2]> first I/O port data high"),
        _ => crate::panic("PS/2 first I/O port test failed"),
    }
    // TODO:
    //   Log the results instead of printing

    if ps2_mode {
        clear_output_buffer();
        match commands::test_second_io_port() {
            commands::response::IO_PORT_TEST_NO_ERROR => controller.mouse = Some(Mouse::default()),
            commands::response::IO_PORT_TEST_CLOCK_LOW => crate::tty::print("[PS/2]> second I/O port clock low"),
            commands::response::IO_PORT_TEST_CLOCK_HIGH => crate::tty::print("[PS/2]> second I/O port clock high"),
            commands::response::IO_PORT_TEST_DATA_LOW => crate::tty::print("[PS/2]> second I/O port data low"),
            commands::response::IO_PORT_TEST_DATA_HIGH => crate::tty::print("[PS/2]> second I/O port data high"),
            _ => crate::panic("PS/2 second I/O port test failed"),
        }
        // TODO:
        //   Log the results instead of printing
    }

    if let Some(keyboard) = controller.keyboard() {
        let mut init_attempts = 0;
        while let Err(err) = keyboard::init(keyboard) {
            if err.resend_requested() || init_attempts < 3 {
                init_attempts += 1;
                continue;
            }
            crate::tty::print("[PS/2]> Failed to initialize keyboard\n");
            break;
            // TODO:
            //   Log instead of printing
        }
    }

    if let Some(mouse) = controller.mouse() {
        let mut init_attempts = 0;
        while let Err(err) = mouse::init(mouse) {
            if err.resend_requested() || init_attempts < 3 {
                init_attempts += 1;
                continue;
            }
            crate::tty::print("[PS/2]> Failed to initialize mouse\n");
            break;
            // TODO:
            //   Log instead of printing
        }
    }

    // re-fetch controller configuration
    cr = controller.command_register();

    // enable the interrupts for the I/O ports
    cr.set_first_io_port_interrupt_enabled(true);
    if ps2_mode {
        cr.set_second_io_port_interrupt_enabled(true);
    }
    controller.apply_command_register(cr);
    clear_output_buffer();
}
