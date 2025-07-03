.section .text

.extern interrupt_handler

.global enable_interrupts
.global disable_interrupts
.global load_idt

.global interrupts_vectors


trap:                                           // trap for the handlers
    push rax                                    // store the general purpose registers on the stack so that the cpu state can be restored after the handling is done. needs to be done manually since pushad is not a valid instruction in 64 bits mode
    push rbx
    push rcx
    push rdx
    push rsi
    push rdi
    push rbp
    push r8
    push r9
    push r10
    push r11
    push r12
    push r13
    push r14
    push r15

    mov rdi, rsp                                // pass the stack pointer address to the handler
    call interrupt_handler

trap_return:
    pop r15                                     // restore the registers in reverse order, to let the cpu resume its previous work. again, needs to be done manually since popad is not a valid instruction in 64 bits mode
    pop r14
    pop r13
    pop r12
    pop r11
    pop r10
    pop r9
    pop r8
    pop rbp
    pop rdi
    pop rsi
    pop rdx
    pop rcx
    pop rbx
    pop rax

    add rsp, 16                                 // adjust the stack pointer (get rid of the error code and interrupt index)
    iretq                                       // use the 64-bit interrupt return instead of normal return, which will pop more data and can return to a different priviledge level

enable_interrupts:
    sti
    ret

disable_interrupts:
    cli
    ret

load_idt:                                       // load the IDT (Interrupt Descriptors Table)
    lidt [rdi]                                  // RDI stores the argument passed to this subroutine
    ret


.altmacro
.macro isr_err idx, max
isr\idx:                                        // subroutine for the interrupts with error code
    push \idx                                   // the interrupt index
    jmp trap
.if \max-\idx
isr_err %\idx+1, max
.endif
.endm

.altmacro
.macro isr_noerr idx, max
isr\idx:                                        // subroutine for the interrupts without error code
    push 0                                      // dummy error code
    push \idx                                   // the interrupt index
    jmp trap
.if \max-\idx
isr_err %\idx+1, max
.endif
.endm

.altmacro
.macro irq_noerr idx, max, num
irq\num:                                        // subroutine for the re-mapped PIC interrupts without error code
    push 0                                      // dummy error code
    push \idx                                   // the interrupt index
    jmp trap
.if \max-\idx
irq_noerr %\idx+1, max, %\num+1
.endif
.endm

                                                // define the subroutines for all 256 available interrupts
isr_noerr 0, 7
isr_err 8, 8
isr_noerr 9, 9                                  // reserved
isr_err 10, 14
isr_noerr 15, 15                                // reserved
isr_noerr 16, 16
isr_err 17, 17
isr_noerr 18, 20
isr_err 21, 21
isr_noerr 22, 31                                // reserved
irq_noerr 32, 39, 0                             // re-mapped master PIC interrupts
irq_noerr 40, 47, 8                             // re-mapped slave PIC interrupts
isr_noerr 48, 255


.section .data

.altmacro
.macro interrupt_vector name, idx, max
    .quad \name\idx                               // address of the subroutine
    .if \max-\idx
    interrupt_vector \name, %\idx+1, max
    .endif
.endm

interrupts_vectors:                             // array of all the interrupts subroutines
    interrupt_vector isr, 0, 31
    interrupt_vector irq, 0, 15
    interrupt_vector isr, 48, 255
