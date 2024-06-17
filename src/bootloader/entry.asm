section .text

extern entry_main
global start

start:
    mov rsp, 0xFFFF800007400000                 ; adjust the kernel stack pointer
    call entry_main

    mov rax, 0xFFFF800007400000                 ; jump to the kernel
    jmp rax

end:
    hlt
    jmp end
