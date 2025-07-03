pub(in crate::drivers::keyboard::ps2) mod set_1;
pub(in crate::drivers::keyboard::ps2) mod set_2;
pub(in crate::drivers::keyboard::ps2) mod set_3;

use crate::drivers::keyboard::ps2::commands;


#[derive(Clone,Copy,PartialEq)]
pub(in crate::drivers::keyboard::ps2)
enum ScancodeSet {
    Set1,
    Set2,
    Set3,
}

impl TryFrom<u8> for ScancodeSet {
    type Error = u8;

    fn try_from(byte:u8) -> Result<Self, Self::Error> {
        match byte {
            commands::SCANCODE_SET_1 => Ok(Self::Set1),
            commands::SCANCODE_SET_2 => Ok(Self::Set2),
            commands::SCANCODE_SET_3 => Ok(Self::Set3),
            _ => Err(byte),
        }
    }
}

impl Into<u8> for ScancodeSet {
    fn into(self) -> u8 {
        match self {
            Self::Set1 => commands::subcommand::WRITE_SCANCODE_SET1,
            Self::Set2 => commands::subcommand::WRITE_SCANCODE_SET2,
            Self::Set3 => commands::subcommand::WRITE_SCANCODE_SET3,
        }
    }
}

impl Default for ScancodeSet {
    fn default() -> Self {
        Self::Set2
    }
}
