use crate::tty::colors::*;

const LINE_SIZE : usize = 160;
const COLUMNS   : usize = 80;
const ROWS      : usize = 25;

const CHAR_NULL       : u8 = 0x00;
const CHAR_BACKSPACE  : u8 = 0x08;
const CHAR_LINEFEED   : u8 = 0x0A;
const CHAR_WHITESPACE : u8 = 0x20;

const ZERO : u8 = 0x00;

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
    /// Creates a `ScreenBuffer`
    const fn new() -> Self {
        ScreenBuffer {
            buf : 0xB8000 as *mut u8,
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
            match *buffer.wrapping_add(i) {
                CHAR_LINEFEED => {
                    self.col = 0;
                    self.row += 1;
                },
                CHAR_BACKSPACE => {
                    if self.is_empty() {
                        continue;
                    }
                    self.decrease_column();
                    *self.buf.wrapping_add(self.col*2 + self.row*LINE_SIZE) = ZERO;
                    *self.buf.wrapping_add(self.col*2 + self.row*LINE_SIZE + 1) = ZERO;
                },
                character => {
                    *self.buf.wrapping_add(self.col*2 + self.row*LINE_SIZE) = character;
                    *self.buf.wrapping_add(self.col*2 + self.row*LINE_SIZE + 1) = color;
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
        //memmove(self.buf, self.buf+LINE_SIZE, LINE_SIZE*24);
        //memset(self.buf+LINE_SIZE*24, 0, LINE_SIZE);
        //self.row -= 1;
        for row in lines..ROWS {
            for col in 0..COLUMNS {
                *self.buf.wrapping_add(col*2 + (row-lines)*LINE_SIZE) = *self.buf.wrapping_add(col*2 + row*LINE_SIZE);
                *self.buf.wrapping_add(col*2 + (row-lines)*LINE_SIZE + 1) = *self.buf.wrapping_add(col*2 + row*LINE_SIZE + 1);
            }
        }
        for row in 1..=lines {
            for col in 0..COLUMNS {
                *self.buf.wrapping_add(col*2 + (ROWS-row)*LINE_SIZE) = CHAR_NULL;
                *self.buf.wrapping_add(col*2 + (ROWS-row)*LINE_SIZE + 1) = CHAR_NULL;
            }
        }
        self.row -= lines;
    }

    /// Zeroes the video memory buffer
    unsafe fn clear(&mut self) {
        //memset(self.buf, 0, self.row*LINE_SIZE);
        for row in 0..ROWS {
            for col in 0..COLUMNS {
                *self.buf.wrapping_add(col*2 + row*LINE_SIZE) = CHAR_NULL;
                *self.buf.wrapping_add(col*2 + row*LINE_SIZE + 1) = FG_BLACK;
            }
        }
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

