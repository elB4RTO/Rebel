.section .text

.global cpuid_address_size


cpuid_address_size:
    push rbp
    mov rbp, rsp

    mov eax, 0x80000008
    cpuid

    pop rbp
    ret
