pub(crate) mod ps2;


pub(crate)
trait Keyboard {
    /// Returns the [`KeyboardType`] associated with the keyboard
    fn keyboard_type(&self) -> KeyboardType;

    /// Handles a _Keyboard Interrupt_
    ///
    /// ## Warning
    ///
    /// This function assumes data is available in the keyboard buffer and
    /// shall not be called manually.
    fn handle_interrupt(&mut self);

    /// Reads data from the keyboard buffer, if any is available
    ///
    /// This function is designed to be manually called by a process, in order
    /// fetch key events when interrupts are disabled for the keyboard, or
    /// masked for the whole system.
    fn fetch(&mut self);

    /// Returns the next [`KeyEvent`] in the queue, if any
    fn next(&mut self) -> Option<KeyEvent>;
}


#[derive(Clone,Copy)]
pub(crate)
enum KeyboardType {
    PS2,
}


#[derive(Clone,Copy)]
pub(crate)
struct KeyEvent {
    pub(crate) key   : Key,
    pub(crate) state : KeyState,
}

impl From<(Key,KeyState)> for KeyEvent {
    fn from((key,state):(Key,KeyState)) -> Self {
        Self { key, state }
    }
}


#[allow(non_camel_case_types)]
#[derive(Clone,Copy)]
pub(crate)
enum Key {
    Typewrite_0,
    Typewrite_1,
    Typewrite_2,
    Typewrite_3,
    Typewrite_4,
    Typewrite_5,
    Typewrite_6,
    Typewrite_7,
    Typewrite_8,
    Typewrite_9,
    Typewrite_A,
    Typewrite_B,
    Typewrite_C,
    Typewrite_D,
    Typewrite_E,
    Typewrite_F,
    Typewrite_G,
    Typewrite_H,
    Typewrite_I,
    Typewrite_J,
    Typewrite_K,
    Typewrite_L,
    Typewrite_M,
    Typewrite_N,
    Typewrite_O,
    Typewrite_P,
    Typewrite_Q,
    Typewrite_R,
    Typewrite_S,
    Typewrite_T,
    Typewrite_U,
    Typewrite_V,
    Typewrite_W,
    Typewrite_X,
    Typewrite_Y,
    Typewrite_Z,
    Typewrite_Dot,              // .
    Typewrite_Comma,            // ,
    Typewrite_Colon,            // :
    Typewrite_Semicolon,        // ;
    Typewrite_Hyphen,           // -
    Typewrite_Underscore,       // _
    Typewrite_Apostrophe,       // '
    Typewrite_Quotes,           // "
    Typewrite_Backtick,         // `
    Typewrite_Caret,            // ^
    Typewrite_Exclamation,      // !
    Typewrite_Question,         // ?
    Typewrite_Plus,             // +
    Typewrite_Asterisk,         // *
    Typewrite_Percent,          // %
    Typewrite_Equal,            // =
    Typewrite_Slash,            // /
    Typewrite_Backslash,        // \
    Typewrite_Pipe,             // |
    Typewrite_Ampersand,        // &
    Typewrite_Dollar,           // $
    Typewrite_Hash,             // #
    Typewrite_Snail,            // @
    Typewrite_Paren,            // (
    Typewrite_Unparen,          // )
    Typewrite_Bracket,          // [
    Typewrite_Unbracket,        // ]
    Typewrite_Brace,            // {
    Typewrite_Unbrace,          // }
    Typewrite_Lesser,           // <
    Typewrite_Greater,          // >
    Typewrite_Tilde,            // ~
    Space,
    Tab,
    Backspace,
    Delete,
    Insert,
    Escape,
    Enter,
    Pause,
    PrintScreen,
    LeftControl,                // Ctrl
    RightControl,               // Ctrl
    LeftMeta,                   // Alt
    RightMeta,                  // AltGr
    LeftSuper,                  // Super
    RightSuper,                 // Super
    LeftShift,
    RightShift,
    CapsLock,
    NumberLock,
    ScrollLock,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    Up,
    Down,
    Left,
    Right,
    PageUp,
    PageDown,
    Home,
    End,
    Keypad_0,
    Keypad_1,
    Keypad_2,
    Keypad_3,
    Keypad_4,
    Keypad_5,
    Keypad_6,
    Keypad_7,
    Keypad_8,
    Keypad_9,
    Keypad_Plus,
    Keypad_Hyphen,
    Keypad_Asterisk,
    Keypad_Slash,
    Keypad_Dot,
    Keypad_Enter,
    Menu,                       // a.k.a. Apps
    Calculator,
    Print,
    Email,
    Folders,                    // a.k.a. MyComputer
    MM_Play,
    MM_Stop,
    MM_NextTrack,
    MM_PreviousTrack,
    MM_MediaSelect,
    MM_Mute,
    MM_VolumeUp,
    MM_VolumeDown,
    WWW_Home,
    WWW_Search,
    WWW_Favorites,
    WWW_Refresh,
    WWW_Stop,
    WWW_Forward,
    WWW_Back,
    ACPI_Power,
    ACPI_Sleep,
    ACPI_Wake,
}


#[derive(Clone,Copy)]
pub(crate)
enum KeyState {
    Pressed,
    Released,
}
