use crate::idt::idt::InterruptFrame;


// the parameter arg represents the stack memory address of the data in user mode
type SystemCall = fn(stack_frame:&mut InterruptFrame);


const N_SYSTEM_CALLS : usize = 64;

#[allow(non_upper_case_globals)]
static mut system_calls : [SystemCall; N_SYSTEM_CALLS] = [syscall::invalid_syscall; N_SYSTEM_CALLS];


pub(in crate::idt) unsafe
fn init_system_calls() {
    system_calls[0] = syscall::mem__total_memory;
    system_calls[1] = syscall::mem__available_memory;
    system_calls[2] = syscall::mem__allocate;
    system_calls[3] = syscall::mem__allocate_zeroed;
    system_calls[4] = syscall::mem__reallocate;
    system_calls[5] = syscall::mem__deallocate;
}

pub(in crate::idt)
fn system_call(stack_frame:&mut InterruptFrame) {
    let syscall_number = stack_frame.rax() as usize; // RAX holds the index number of the syscall

    if syscall_number >= N_SYSTEM_CALLS {
        stack_frame.set_rax(retcode::INVALID_SYSCALL);
        return;
    }

    unsafe { system_calls[syscall_number](stack_frame); }
}


mod retcode {

    pub(super) const
    INVALID_SYSCALL : u64 = u64::MAX;

    pub(super) const
    OK : u64 = 0;

    pub(super) const
    ERR : u64 = u64::MAX - 1;

} // mod retcode

mod syscall {
    #![allow(non_snake_case)]

    use super::retcode;

    use crate::Log;
    use crate::idt::idt::InterruptFrame;
    use crate::memory::{self, MemoryOwner};

    const MEM_OWNER : MemoryOwner = MemoryOwner::User;

    pub(super)
    fn invalid_syscall(stack_frame:&mut InterruptFrame) {
        stack_frame.set_rax(retcode::INVALID_SYSCALL);
    }

    pub(super)
    fn mem__total_memory(stack_frame:&mut InterruptFrame) {
        stack_frame.set_rax(memory::total_space());
    }

    pub(super)
    fn mem__available_memory(stack_frame:&mut InterruptFrame) {
        stack_frame.set_rax(memory::available_space());
    }

    pub(super)
    fn mem__allocate(stack_frame:&mut InterruptFrame) {
        let size = stack_frame.rdi();
        match memory::alloc(size, MEM_OWNER) {
            Ok(addr) => stack_frame.set_rax(addr),
            Err(e) => e.log_and_then(|| stack_frame.set_rax(retcode::ERR)),
        }
    }

    pub(super)
    fn mem__allocate_zeroed(stack_frame:&mut InterruptFrame) {
        let size = stack_frame.rdi();
        match memory::zalloc(size, MEM_OWNER) {
            Ok(addr) => stack_frame.set_rax(addr),
            Err(e) => e.log_and_then(|| stack_frame.set_rax(retcode::ERR)),
        }
    }

    pub(super)
    fn mem__reallocate(stack_frame:&mut InterruptFrame) {
        let old_addr = stack_frame.rdi();
        let new_size = stack_frame.rsi();
        match memory::realloc(old_addr, new_size, MEM_OWNER) {
            Ok(new_addr) => stack_frame.set_rax(new_addr),
            Err(e) => e.log_and_then(|| stack_frame.set_rax(retcode::ERR)),
        }
    }

    pub(super)
    fn mem__deallocate(stack_frame:&mut InterruptFrame) {
        let addr = stack_frame.rdi();
        match memory::dealloc(addr, MEM_OWNER) {
            Ok(()) => stack_frame.set_rax(retcode::OK),
            Err(e) => e.log_and_then(|| stack_frame.set_rax(retcode::ERR)),
        }
    }

} // mod syscall
