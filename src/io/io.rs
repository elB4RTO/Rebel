use core::arch::global_asm;


pub(crate) const
PIC_MASTER_COMMAND_PORT : u16 = 0x0020;
pub(crate) const
PIC_MASTER_DATA_PORT    : u16 = 0x0021;
pub(crate) const
PIC_SLAVE_COMMAND_PORT  : u16 = 0x00A0;
pub(crate) const
PIC_SLAVE_DATA_PORT     : u16 = 0x00A1;

pub(crate) const
PS2_DATA_PORT           : u16 = 0x0060;
pub(crate) const
PS2_REGISTER_PORT       : u16 = 0x0064;


global_asm!(include_str!("io.asm"));

unsafe extern "C" {
    /// Reads one Byte from the given I/O port
    pub(crate)
    fn in_byte(port:u16) -> u8;

    /// Reads one Word from the given I/O port
    pub(crate)
    fn in_word(port:u16) -> u16;

    /// Reads one DoubleWord from the given I/O port
    pub(crate)
    fn in_double_word(port:u16) -> u32;

    /// Writes one Byte to the given I/O port
    pub(crate)
    fn out_byte(port:u16, value:u8);

    /// Writes one Word to the given I/O port
    pub(crate)
    fn out_word(port:u16, value:u16);

    /// Writes one DoubleWord to the given I/O port
    pub(crate)
    fn out_double_word(port:u16, value:u32);
}
