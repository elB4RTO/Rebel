use super::colors::*;
use crate::memory::{memset,memcpy};

const VIDEO_MEM         : u64   = 0xB8000;

const LINE_SIZE         : usize = 160;
const COLUMNS           : usize = 80;
const ROWS              : usize = 25;

const CHAR_NULL         : u8    = 0x00;
const CHAR_BACKSPACE    : u8    = 0x08;
const CHAR_LINEFEED     : u8    = 0x0A;
const CHAR_WHITESPACE   : u8    = 0x20;

const ZERO              : u8    = 0x00;

const HEX_MAP : [u8;16] = [0x30,0x31,0x32,0x33,0x34,0x35,0x36,0x37,0x38,0x39,0x41,0x42,0x43,0x44,0x45,0x46];
const DEC_MAP : [u8;10] = [0x30,0x31,0x32,0x33,0x34,0x35,0x36,0x37,0x38,0x39];

#[allow(non_upper_case_globals)]
static mut screen_buffer : ScreenBuffer = ScreenBuffer::new();

/// This struct is used to interface with the video memory buffer
///
/// `buf` points to the base address of the video memory.
/// `col` and `row` keep track of the on-screen position of the cursor
/// and are used to compute the offset in the memory buffer.
struct ScreenBuffer {
    buf : *mut u8,
    col : usize,
    row : usize,
}

unsafe impl Send for ScreenBuffer {}

unsafe impl Sync for ScreenBuffer {}

impl ScreenBuffer {
    /// Creates a new [`ScreenBuffer`]
    const fn new() -> Self {
        ScreenBuffer {
            buf : VIDEO_MEM as *mut u8,
            col : 0_usize,
            row : 0_usize,
        }
    }

    /// Checks whether both `col` and `row` equal 0
    fn is_empty(&self) -> bool {
        (self.col == 0) & (self.row == 0)
    }

    /// Increments `col` by 1
    ///
    /// If `col` points to the end of the line, it is set to 0 and `row`
    /// is incremented by 1.
    /// This function doesn't check whether incrementing `row` leaves it in a
    /// valid position or not (namely whether `row` exceeds the number of rows).
    fn increase_column(&mut self) {
        self.col += 1;
        if self.col == COLUMNS {
            self.col = 0;
            self.row += 1;
        }
    }

    /// Decreases `col` by 1
    ///
    /// If `col` is 0, it is set to the end of the line and `row` is decremented by 1.
    /// This function doesn't check whether decrementing `row` may cause an underflow.
    fn decrease_column(&mut self) {
        if self.col == 0 {
            self.col = COLUMNS;
            self.row -= 1;
        }
        self.col -= 1;
    }

    /// Copies the content of `buffer` inside the video memory
    ///
    /// This function also parses `buffer` for special characters such as
    /// LineFeed, BackSpace, etc..., to provide text-mode funcionalities
    unsafe fn write(&mut self, buffer:*const u8, size:usize, color:u8) {
        for i in 0..size {
            match *buffer.add(i) {
                CHAR_LINEFEED => {
                    self.col = 0;
                    self.row += 1;
                },
                CHAR_BACKSPACE => {
                    if self.is_empty() {
                        continue;
                    }
                    self.decrease_column();
                    *self.buf.add(self.col*2 + self.row*LINE_SIZE) = ZERO;
                    *self.buf.add(self.col*2 + self.row*LINE_SIZE + 1) = ZERO;
                },
                character => {
                    *self.buf.add(self.col*2 + self.row*LINE_SIZE) = character;
                    *self.buf.add(self.col*2 + self.row*LINE_SIZE + 1) = color;
                    self.increase_column();
                },
            }
            if self.row == ROWS {
                self.scroll(1);
            }
        }
    }

    /// Moves the memory in the buffer to simulate a screen scroll
    unsafe fn scroll(&mut self, lines:usize) {
        if lines >= self.row || lines >= ROWS {
            self.clear();
            return;
        }
        let scroll_size = lines * LINE_SIZE;
        let scrolled_size = (self.row - lines) * LINE_SIZE;
        memcpy(self.buf as u64, self.buf.add(scroll_size) as u64, scrolled_size as u64);
        memset(self.buf.add(scrolled_size) as u64, ZERO, scroll_size as u64);
        self.row -= lines;
    }

    /// Zeroes the video memory buffer
    unsafe fn clear(&mut self) {
        memset(self.buf as u64, ZERO, (ROWS*LINE_SIZE) as u64);
        self.col = 0;
        self.row = 0;
    }
}

/// Prints `msg` on screen
pub(crate) fn print(msg:&str) {
    unsafe {
        screen_buffer.write(msg.as_ptr(), msg.len(), FG_WHITE);
    }
}

/// Prints `msg` on screen
pub(crate) fn print_raw(msg:&[u8]) {
    unsafe {
        screen_buffer.write(msg.as_ptr(), msg.len(), FG_WHITE);
    }
}

/// Prints `value` on screen in hexadecimal format
pub(crate) fn print_hex(mut value:usize) {
    let mut tmp_buffer : [u8;32] = [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0];
    let mut size = 0_usize;

    while value != 0 {
        tmp_buffer[size] = HEX_MAP[value % 16];
        value /= 16;
        size += 1;
    }
    if size == 0 {
        tmp_buffer[size] = 0x30;
        size += 1;
    }

    let mut buffer : [u8;33] = [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0];
    buffer[0] = 0x78;
    for i in 1..=size {
        buffer[i] = tmp_buffer[size-i] as u8;
    }
    unsafe {
        screen_buffer.write(buffer.as_ptr(), size+1, FG_WHITE);
    }
}

/// Prints `value` on screen
pub(crate) fn print_usize(mut value:usize) {
    let mut tmp_buffer : [u8;32] = [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0];
    let mut size = 0_usize;

    while value != 0 {
        tmp_buffer[size] = DEC_MAP[value % 10];
        value /= 10;
        size += 1;
    }
    if size == 0 {
        tmp_buffer[size] = 0x30;
        size += 1;
    }

    let mut buffer : [u8;32] = [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0];
    for i in 0..size {
        buffer[i] = tmp_buffer[size-i-1] as u8;
    }
    unsafe {
        screen_buffer.write(buffer.as_ptr(), size, FG_WHITE);
    }
}

/// Prints `value` on screen in hexadecimal format
pub(crate) fn print_isize(mut value:isize) {
    if value < 0 {
        print("-");
        value *= -1;
    }
    print_usize(value as usize);
}

/// Clears the entire screen
pub(crate) fn clear() {
    unsafe {
        screen_buffer.clear();
    }
}

/// Scrolls the view by a number of lines defined by `lines`
pub(crate) fn scroll(lines:usize) {
    unsafe {
        screen_buffer.scroll(lines);
    }
}
