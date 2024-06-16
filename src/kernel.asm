section .text

extern kernel_main
global start

start:
    mov rax, GDT64Ptr                           ; cannot directly reference the GDT pointer with a 32-bits pointer
    lgdt [rax]

set_tss:
    mov rax, TSS
    mov rdi, TSSDesc
    mov [rdi+2], ax                             ; the lower 16 bits of the address are in the third Byte of the TSS descriptor
    shr rax, 16                                 ; by shifting right 16 bits, the value of bits from 16 to 23 are now in AL
    mov [rdi+4], al
    shr rax, 8                                  ; move bits from 24 to 31
    mov [rdi+7], al
    shr rax, 8                                  ; move bits from 32 to 63
    mov [rdi+8], eax

    mov ax, 0x20
    ltr ax                                      ; load task register (5th entry of the GDT)

remap_pic:                                      ; Programmable Interrupt Controller
    mov al, 00010001b                           ; b0=1: need 4th init step, b1=0: cascade, b3=0: edge, b4=1: init
    out 0x20, al                                ; 0x20 is the address of the command register
    out 0xA0, al                                ; 0xA0 is the address of the slave

    mov al, 0x20                                ; set master IRQ0 to 32
    out 0x21, al
    mov al, 40                                  ; set slave IRQ0 to 40
    out 0xA1, al

    mov al, 00000100b                           ; the slave is attached to the master via IRQ2 (if the 3rd is set it means that the IRQ2 is used)
    out 0x21, al
    mov al, 00000010b                           ; for the slave is the 2nd bit
    out 0xA1, al

    mov al, 00000001b                           ; select the mode: b0=1: x86 mode, b1=0: no AEOI (automatic end of interrupt), b2-3=0: don't use buffered mode, b4=0: don't use fully nested mode
    out 0x21, al
    out 0xA1, al

    mov al, 11111100b                           ; mask all the IRQs of the master except IRQ0 and IRQ1 (which is for the keyboard)
    out 0x21, al
    mov al, 11111111b                           ; mask all the IRQs of the slave
    out 0xA1, al

    mov rax, kernel_entry                       ; cannot directly push the address of the kernel memory (since it is 64 bit and is only possible to push 32 bits immediate values)
    push 8                                      ; the code segment is the second entry of GDT (8 Bytes each)
    push rax
    db 0x48                                     ; the default operand size of RETF is 32 bits, but the data pushed is 64 bits, so override the operand size with a prefix of 48 to change it to 64 bits
    retf                                        ; use Far Return to load the code segment descriptor into the CS register (normal Return won't do that)

kernel_entry:
    xor ax, ax
    mov ss, ax                                  ; zero the SS register to handle interrupts without causing exceptions

    mov rsp, 0xFFFF800007400000                 ; adjust the kernel stack pointer address
    call kernel_main

    sti                                         ; interrupts were disabled while switching to long mode

end:
    hlt
    jmp end


section .data

global TSS

GDT64:                                          ; same as per loader.asm
    dq 0
    dq 0x0020980000000000
    dq 0x0020f80000000000                       ; the code segment descriptors for ring 3
    dq 0x0000f20000000000
TSSDesc:                                        ; the TSS descriptor
    dw TSSLen-1
    dw 0                                        ; set the base address to 0 for the moment, the actual address will be assigned in the code
    db 0
    db 0x89
    db 0
    db 0
    dq 0

GDT64Len: equ $-GDT64
GDT64Ptr: dw GDT64Len-1
          dq GDT64

TSS:                                            ; Task State Descriptor
    dd 0
    dq 0xFFFF800007400000                       ; the address of the RSP
    times 88 db 0                               ; unused fields
    dd TSSLen                                   ; address of IO permission bitmap. assign the size of TSS since unused

TSSLen: equ $-TSS
