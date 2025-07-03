pub(crate) mod ps2;


use core::ops::BitAnd;


pub(crate)
trait Mouse {
    /// Returns the [`MouseType`] associated with the mouse
    fn mouse_type(&self) -> MouseType;

    /// Handles a _Mouse Interrupt_
    ///
    /// ## Warning
    ///
    /// This function assumes data is available in the mouse buffer and
    /// shall not be called manually.
    fn handle_interrupt(&mut self);

    /// Reads data from the mouse buffer, if any is available
    ///
    /// This function is designed to be manually called by a process, in order
    /// fetch events when interrupts are disabled for the mouse, or masked for
    /// the whole system.
    fn fetch(&mut self);

    /// Returns the next [`MouseEvent`] in the queue, if any
    fn next(&mut self) -> Option<MouseEvent>;
}


#[derive(Clone,Copy)]
pub(crate)
enum MouseType {
    PS2,
}


#[derive(Clone,Copy)]
pub(crate)
struct MouseEvent {
    pub(crate) buttons    : u16,
    pub(crate) x_movement : i16,
    pub(crate) y_movement : i16,
    pub(crate) z_movement : i16,
}

impl MouseEvent {
    pub(crate)
    fn button_1_pressed(&self) -> bool {
        Button::Button1 == (Button::Button1 & self.buttons)
    }

    pub(crate)
    fn button_2_pressed(&self) -> bool {
        Button::Button2 == (Button::Button2 & self.buttons)
    }

    pub(crate)
    fn button_3_pressed(&self) -> bool {
        Button::Button3 == (Button::Button3 & self.buttons)
    }
}


#[repr(u16)]
#[derive(Clone,Copy)]
pub(crate)
enum Button {
    Button1  = 0b0000000000000001,
    Button2  = 0b0000000000000010,
    Button3  = 0b0000000000000100,
    Button4  = 0b0000000000001000,
    Button5  = 0b0000000000010000,
    Button6  = 0b0000000000100000,
    Button7  = 0b0000000001000000,
    Button8  = 0b0000000010000000,
    Button9  = 0b0000000100000000,
    Button10 = 0b0000001000000000,
    Button11 = 0b0000010000000000,
    Button12 = 0b0000100000000000,
    Button13 = 0b0001000000000000,
    Button14 = 0b0010000000000000,
    Button15 = 0b0100000000000000,
}

impl BitAnd<u16> for Button {
    type Output = u16;

    fn bitand(self, rhs:u16) -> Self::Output {
        self as u16 & rhs
    }
}

impl PartialEq<u16> for Button {
    fn eq(&self, other:&u16) -> bool {
        *self as u16 == *other
    }
}
