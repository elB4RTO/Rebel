use crate::idt::isr::*;
use crate::idt::syscall::*;

use core::arch::global_asm;

global_asm!(include_str!("idt.asm"));

unsafe extern "C" {
    /// The array of interrupts' vectors
    static interrupts_vectors : [u64; N_INTERRUPTS];

    /// Informs the processor the interrupt has been handled
    pub(in crate::idt)
    fn aknowledge_interrupt();

    /// Enables the interrupts
    pub(in crate::idt)
    fn enable_interrupts();

    /// Disables the interrupts
    pub(in crate::idt)
    fn disable_interrupts();

    /// Loads the given IDT
    fn load_idt(ptr:*const IDTR);

    pub(in crate::idt)
    fn read_isr() -> u8;
}


type InterruptCallback = fn(&mut InterruptFrame);


pub(in crate::idt)
const N_INTERRUPTS : usize = 256;

#[allow(non_upper_case_globals)]
static mut idtr : IDTR = IDTR::new();
#[allow(non_upper_case_globals)]
static mut interrupts_descriptors : [GateDescriptor; N_INTERRUPTS] = [GateDescriptor::new(); N_INTERRUPTS];
#[allow(non_upper_case_globals)]
static mut interrupts_callbacks : [InterruptCallback; N_INTERRUPTS] = [unexpected_isr; N_INTERRUPTS];

const RING0_ATTRIBUTES : u8 = 0x8E; // 1_00_0_1110
const RING3_ATTRIBUTES : u8 = 0xEE; // 1_11_0_1110


/// Represents the structure of each entry of the IDT
#[derive(Clone,Copy)]
#[repr(C,packed)]
struct GateDescriptor {
    offset_0_15      : u16, //  0~15  > offset 0~15
    segment_selector : u16, // 16~31  > segment selector
    ist_reserved     : u8,  // 32~39  > IST (32~34) + reserved (35~39)
    attributes       : u8,  // 40~47  > gate type (40~43) + zero (44) + DPL (45~46) + present bit (47)
    offset_16_31     : u16, // 48~63  > offset 16~31
    offset_32_63     : u32, // 64~95  > offset 32~63
    reserved         : u32, // 96~127 > reserved
}

impl GateDescriptor {
    const
    fn new() -> Self {
        Self {
            offset_0_15      : 0x0000,
            segment_selector : 0x0000,
            ist_reserved     : 0x00,
            attributes       : 0x00,
            offset_16_31     : 0x0000,
            offset_32_63     : 0x00000000,
            reserved         : 0x00000000,
        }
    }
}


/// Used to load IDT instructions
#[repr(C,packed)]
struct IDTR {
    limit : u16, // first 2 Bytes are the limit
    addr  : u64, // address of the IDT
}

impl IDTR {
    const
    fn new() -> Self {
        Self {
            limit : 0x0000,
            addr  : 0x0000000000000000,
        }
    }
}


// Represents the state of the stack, at the moment an ISR handler is called
#[repr(C,packed)]
pub(in crate::idt)
struct InterruptFrame {
    r15     : u64, // general purpose register
    r14     : u64, // general purpose register
    r13     : u64, // general purpose register
    r12     : u64, // general purpose register
    r11     : u64, // general purpose register
    r10     : u64, // general purpose register
    r9      : u64, // general purpose register
    r8      : u64, // general purpose register
    rbp     : u64, // general purpose register
    rdi     : u64, // general purpose register
    rsi     : u64, // general purpose register
    rdx     : u64, // general purpose register
    rcx     : u64, // general purpose register
    rbx     : u64, // general purpose register
    rax     : u64, // general purpose register
    intno   : u64, // interrupt index
    errcode : u64, // sometimes pushed by the cpu, sometimes manually
    rip     : u64, // automatically pushed by the processor
    cs      : u64, // automatically pushed by the processor
    rflags  : u64, // automatically pushed by the processor
    rsp     : u64, // automatically pushed by the processor
    ss      : u64, // automatically pushed by the processor
}

impl InterruptFrame {
    pub(in crate::idt)
    fn rdi(&self) -> u64 {
        self.rdi
    }

    pub(in crate::idt)
    fn rsi(&self) -> u64 {
        self.rsi
    }

    pub(in crate::idt)
    fn rax(&self) -> u64 {
        self.rax
    }

    pub(in crate::idt)
    fn set_rax(&mut self, value:u64) {
        self.rax = value;
    }

    pub(in crate::idt)
    fn interrupt_number(&self) -> u64 {
        self.intno
    }

    pub(in crate::idt)
    fn error_code(&self) -> u64 {
        self.errcode
    }
}


fn size_of_interrupts_descriptors() -> usize {
    (core::mem::size_of::<GateDescriptor>() * N_INTERRUPTS) - 1
}

// Initializes and loads the IDT
pub(crate)
fn init_idt() {
    crate::tty::print("[IDT]> Initializing interrupts handlers\n");

    unsafe {
        init_isr_callbacks();
        init_system_calls();

        for i in 0..N_INTERRUPTS {
            init_idt_entry(i, RING0_ATTRIBUTES);
        }
        init_idt_entry(128, RING3_ATTRIBUTES); // only system calls are allowed in userland

        idtr.limit = size_of_interrupts_descriptors() as u16;
        idtr.addr = &raw const interrupts_descriptors as u64;

        load_idt(&raw const idtr);
    }
}

/// Initializes the IDT entry at the given index
unsafe
fn init_idt_entry(entry:usize, attributes:u8) {
    let addr = interrupts_vectors[entry];
    let ref mut descr = interrupts_descriptors[entry];
    descr.offset_0_15 = addr as u16;
    descr.segment_selector = 8; // set to the kernel code segment selector
    descr.attributes = attributes;
    descr.offset_16_31 = (addr >> 16) as u16;
    descr.offset_32_63 = (addr >> 32) as u32;
}

/// Initializes the array of the interrupt handlers callbacks
unsafe
fn init_isr_callbacks() {
    interrupts_callbacks[0] = isr0;
    interrupts_callbacks[1] = isr1;
    interrupts_callbacks[2] = isr2;
    interrupts_callbacks[3] = isr3;
    interrupts_callbacks[4] = isr4;
    interrupts_callbacks[5] = isr5;
    interrupts_callbacks[6] = isr6;
    interrupts_callbacks[7] = isr7;
    interrupts_callbacks[8] = isr8;
    interrupts_callbacks[10] = isr10;
    interrupts_callbacks[11] = isr11;
    interrupts_callbacks[12] = isr12;
    interrupts_callbacks[13] = isr13;
    interrupts_callbacks[14] = isr14;
    interrupts_callbacks[16] = isr16;
    interrupts_callbacks[17] = isr17;
    interrupts_callbacks[18] = isr18;
    interrupts_callbacks[19] = isr19;
    interrupts_callbacks[20] = isr20;
    interrupts_callbacks[21] = isr21;
    interrupts_callbacks[32] = isr32;
    interrupts_callbacks[33] = isr33;
    interrupts_callbacks[39] = isr39;
    interrupts_callbacks[128] = isr128;
}


/// Handles interrupts
#[no_mangle]
unsafe extern "C"
fn interrupt_handler(stack_frame:&mut InterruptFrame) {
    let interrupt_number = stack_frame.intno as usize;
    if interrupt_number >= N_INTERRUPTS {
        crate::panic("Interrupt number out of bounds");
    }

    // TODO:
    //   Store the current process registers and pages,
    //   than load the kernel's registers and pages

    interrupts_callbacks[interrupt_number](stack_frame);

    // TODO:
    //   Restore the current process' registers and pages
}
