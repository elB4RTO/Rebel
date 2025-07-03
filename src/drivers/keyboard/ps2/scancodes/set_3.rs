use crate::drivers::keyboard::{KeyEvent, Key, KeyState};


const
MAX_SCANCODE_LEN : usize = 1;


pub(in crate::drivers::keyboard::ps2)
fn key_from(buf:&[u8]) -> Result<Option<KeyEvent>, ()> {
    if buf.len() > MAX_SCANCODE_LEN {
        return Err(());
    }

    match buf {
        [0x1C] => Ok(Some((Key::Typewrite_A, KeyState::Pressed).into())),
        [0x32] => Ok(Some((Key::Typewrite_B, KeyState::Pressed).into())),
        [0x21] => Ok(Some((Key::Typewrite_C, KeyState::Pressed).into())),
        [0x23] => Ok(Some((Key::Typewrite_D, KeyState::Pressed).into())),
        [0x24] => Ok(Some((Key::Typewrite_E, KeyState::Pressed).into())),
        [0x2B] => Ok(Some((Key::Typewrite_F, KeyState::Pressed).into())),
        [0x34] => Ok(Some((Key::Typewrite_G, KeyState::Pressed).into())),
        [0x33] => Ok(Some((Key::Typewrite_H, KeyState::Pressed).into())),
        [0x43] => Ok(Some((Key::Typewrite_I, KeyState::Pressed).into())),
        [0x3B] => Ok(Some((Key::Typewrite_J, KeyState::Pressed).into())),
        [0x42] => Ok(Some((Key::Typewrite_K, KeyState::Pressed).into())),
        [0x4B] => Ok(Some((Key::Typewrite_L, KeyState::Pressed).into())),
        [0x3A] => Ok(Some((Key::Typewrite_M, KeyState::Pressed).into())),
        [0x31] => Ok(Some((Key::Typewrite_N, KeyState::Pressed).into())),
        [0x44] => Ok(Some((Key::Typewrite_O, KeyState::Pressed).into())),
        [0x4D] => Ok(Some((Key::Typewrite_P, KeyState::Pressed).into())),
        [0x15] => Ok(Some((Key::Typewrite_Q, KeyState::Pressed).into())),
        [0x2D] => Ok(Some((Key::Typewrite_R, KeyState::Pressed).into())),
        [0x1B] => Ok(Some((Key::Typewrite_S, KeyState::Pressed).into())),
        [0x2C] => Ok(Some((Key::Typewrite_T, KeyState::Pressed).into())),
        [0x3C] => Ok(Some((Key::Typewrite_U, KeyState::Pressed).into())),
        [0x2A] => Ok(Some((Key::Typewrite_V, KeyState::Pressed).into())),
        [0x1D] => Ok(Some((Key::Typewrite_W, KeyState::Pressed).into())),
        [0x22] => Ok(Some((Key::Typewrite_X, KeyState::Pressed).into())),
        [0x35] => Ok(Some((Key::Typewrite_Y, KeyState::Pressed).into())),
        [0x1A] => Ok(Some((Key::Typewrite_Z, KeyState::Pressed).into())),
        _ => Err(()),
    }
}
