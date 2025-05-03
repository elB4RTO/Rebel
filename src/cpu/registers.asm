.section .text

.global get_cr0
.global set_cr0
.global get_cr2
.global set_cr2
.global get_cr3
.global set_cr3
.global get_cr4
.global set_cr4
.global get_cr8
.global set_cr8
.global get_msr
.global set_msr


get_cr0:
    mov rax, cr0
    ret

set_cr0:
    mov cr0, rdi
    ret


get_cr2:
    mov rax, cr2
    ret

set_cr2:
    mov cr2, rdi
    ret


get_cr3:
    mov rax, cr3
    ret

set_cr3:
    mov cr3, rdi
    ret


get_cr4:
    mov rax, cr4
    ret

set_cr4:
    mov cr4, rdi
    ret

get_cr8:
    mov rax, cr1
    ret

set_cr8:
    mov cr1, rdi
    ret

get_msr:
    mov rcx, rdi
    rdmsr
    ret

set_msr:
    mov rcx, rdi
    mov rax, rsi
    wrmsr
    ret
