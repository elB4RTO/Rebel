
pub(crate) fn start() {
    clear();
    print("Welcome in the kernel");
    loop {}
}


#[panic_handler]
fn panic(msg:&core::panic::PanicInfo) -> ! {
    loop {}
}


const LINE_SIZE : usize = 160;
const COLUMNS : usize = 80;
const ROWS : usize = 25;

const FG_BLACK : u8 = 0x00;
const FG_WHITE : u8 = 0x0F;

const CHAR_NULL : u8 = 0x00;
const CHAR_BACKSPACE : u8 = 0x08;
const CHAR_LINEFEED : u8 = 0x0A;
const CHAR_WHITESPACE : u8 = 0x20;

#[allow(non_upper_case_globals)]
static mut screen_buffer : ScreenBuffer = ScreenBuffer::new();

struct ScreenBuffer {
    buf: *mut u8,
    col: usize,
    row: usize,
}

unsafe impl Send for ScreenBuffer {}

unsafe impl Sync for ScreenBuffer {}

impl ScreenBuffer {
    const fn new() -> Self {
        ScreenBuffer {
            buf: 0xB8000 as *mut u8,
            col: 0_usize,
            row: 0_usize,
        }
    }

    unsafe fn write(&mut self, buffer:*const u8, size:usize, color:u8) {
        for i in 0..size {
            if *buffer.wrapping_add(i) == CHAR_LINEFEED {
                self.col = 0;
                self.row += 1;
            } else if *buffer.wrapping_add(i) == CHAR_BACKSPACE {
                if self.col == 0 && self.row == 0 {
                    continue;
                } else if self.col == 0 {
                    self.col = COLUMNS;
                    self.row -= 1;
                }
                self.col -= 1;
                *self.buf.wrapping_add(self.col*2 + self.row*LINE_SIZE) = CHAR_NULL;
                *self.buf.wrapping_add(self.col*2 + self.row*LINE_SIZE + 1) = CHAR_NULL;
            } else {
                *self.buf.wrapping_add(self.col*2 + self.row*LINE_SIZE) = *buffer.wrapping_add(i);
                *self.buf.wrapping_add(self.col*2 + self.row*LINE_SIZE + 1) = color;

                self.col += 1;

                if self.col >= COLUMNS {
                    self.col = 0;
                    self.row += 1;
                }
            }

            if self.row >= ROWS {
                self.scroll(1);
            }
        }
    }

    /// scrolls the view by a number of lines defined by `lines`, leaving the last line empty and free to use
    unsafe fn scroll(&mut self, lines:usize) {
        //memmove(self.buf, self.buf+LINE_SIZE, LINE_SIZE*24);
        //memset(self.buf+LINE_SIZE*24, 0, LINE_SIZE);
        //self.row -= 1;
    }

    unsafe fn clear(&mut self)
    {
        for row in 0..ROWS {
            for col in 0..COLUMNS {
                *self.buf.wrapping_add(col*2 + row*LINE_SIZE) = CHAR_WHITESPACE;
                *self.buf.wrapping_add(col*2 + row*LINE_SIZE + 1) = FG_BLACK;
            }
        }

        self.col = 0;
        self.row = 0;
    }
}

fn print(msg:&str) {
    unsafe { screen_buffer.write(msg.as_ptr(), msg.len(), FG_WHITE) }
}

fn clear() {
    unsafe { screen_buffer.clear(); }
}
