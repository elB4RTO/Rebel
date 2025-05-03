use core::arch::global_asm;


global_asm!(include_str!("io.asm"));

unsafe extern "C" {
    /// Reads one Byte from the given I/O port
    pub(crate)
    fn in_byte(port:u64) -> u64;

    /// Reads one Word from the given I/O port
    pub(crate)
    fn in_word(port:u64) -> u64;

    /// Writes one Byte to the given I/O port
    pub(crate)
    fn out_byte(port:u64, value:u64);

    /// Writes one Word to the given I/O port
    pub(crate)
    fn out_word(port:u64, value:u64);
}
