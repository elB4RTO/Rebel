section .text

extern kernel_main
global start

start:
    mov rax, GDT64Ptr                           ; cannot directly reference the GDT pointer with a 32-bits pointer
    lgdt [rax]

set_tss:
    mov rax, TSS
    mov rdi, TSSDesc
    mov [rdi+2], ax                             ; base 0~15
    shr rax, 16
    mov [rdi+4], al                             ; base 16~23
    shr rax, 8
    mov [rdi+7], al                             ; base 24~31
    shr rax, 8
    mov [rdi+8], eax                            ; base 32~63

    mov ax, 0x28
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

    mov rsp, 0xFFFFFFFF80000000                 ; adjust the kernel stack pointer address
    call kernel_main

    sti                                         ; interrupts were disabled while switching to long mode

end:
    hlt
    jmp end


section .data

global TSS

GDT64:
GDT64Null:
    dq 0x0000000000000000
GDT64CodeK:
    dw 0x0000                                   ;  0~15 > limit (unused)
    dw 0x0000                                   ; 16~31 > base  0~15 (unused)
    db 0x00                                     ; 32~39 > base 16~23 (unused)
    db 0x9A                                     ; 40~47 > access (10011010)
    db 0xA0                                     ; 48~55 > flags (1010) + limit (unused)
    db 0x00                                     ; 56~63 > base 24~31 (unused)
GDT64DataK:
    dw 0x0000                                   ;  0~15 > limit (unused)
    dw 0x0000                                   ; 16~31 > base  0~15 (unused)
    db 0x00                                     ; 32~39 > base 16~23 (unused)
    db 0x92                                     ; 40~47 > access (10010010)
    db 0xC0                                     ; 48~55 > flags (1100) + limit (unused)
    db 0x00                                     ; 56~63 > base 24~31 (unused)
GDT64CodeU:
    dw 0x0000                                   ;  0~15 > limit (unused)
    dw 0x0000                                   ; 16~31 > base  0~15 (unused)
    db 0x00                                     ; 32~39 > base 16~23 (unused)
    db 0xFA                                     ; 40~47 > access (11111010)
    db 0xA0                                     ; 48~55 > flags (1010) + limit (unused)
    db 0x00                                     ; 56~63 > base 24~31 (unused)
GDT64DataU:
    dw 0x0000                                   ;  0~15 > limit (unused)
    dw 0x0000                                   ; 16~31 > base  0~15 (unused)
    db 0x00                                     ; 32~39 > base 16~23 (unused)
    db 0xF2                                     ; 40~47 > access (11110010)
    db 0xC0                                     ; 48~55 > flags (1100) + limit (unused)
    db 0x00                                     ; 56~63 > base 24~31 (unused)
TSSDesc:
    dw TSSLen-1                                 ;  0~15  > limit (size of the TSS)
    dw 0x0000                                   ; 16~31  > base  0~15 (set to 0 for the moment, the actual address will be assigned in the code)
    db 0x00                                     ; 32~39  > base 16~23 (will be filled later)
    db 0x89                                     ; 40~47  > access (10001001)
    db 0x40                                     ; 48~55  > flags (0100) + limit (unused)
    db 0x00                                     ; 56~63  > base 24~31 (will be filled later)
    dd 0x00000000                               ; 64~95  > base 32~63 (will be filled later)
    dd 0x00000000                               ; 96~127 > reserved

GDT64Len: equ $-GDT64                           ; 4 Bytes padding
GDT64Ptr: dw GDT64Len-1
          dq GDT64

TSS:
    dd 0x00000000                               ; reserved
    dq 0xFFFFFFFF80000000                       ; RSP0
    times 88 db 0                               ; unused for the moment
    dd TSSLen                                   ; offset of the IO Permission Bitmap (unused, set to the size of TSS itself)

TSSLen: equ $-TSS

