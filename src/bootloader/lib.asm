section .text

global memset
global memmove
global memcmp


memset:                                         ; arguments are stored in RDI (void* buffer), RSI (unsigned char value), RDX (unsigned int size)
    cld                                         ; clear the direction flag to data from low memory address to high memory address
    mov ecx, edx
    mov al, sil
    rep stosb                                   ; Repeat Store-Byte, copies the value in AL to the memory addressed by RDI
    ret


memmove:                                        ; arguments are stored in RDI (void* dst), RSI (void* src), RDX (unsigned int size)
    cld                                         ; clear the direction flag to data from low memory address to high memory address
    cmp rsi, rdi                                ; compare the two locations to avoid overwriting memory in case the two locations overlap: if destination is greater than source, copy backward, otherwise copy forward
    jae .copy
    mov r8, rsi                                 ; copy the src pointer
    add r8, rdx                                 ; add the size
    cmp r8, rdi                                 ; compare with the dst pointer
    jbe .copy                                   ; check if the address is less or equal to the dst address
.overlap:
    std                                         ; set the direction flag to copy the data from high memory address to low memory address
    add rdi, rdx                                ; data will be copied backward, so increment by the size to start from the end
    add rsi, rdx
    sub rdi, 1                                  ; decrement by 1 to not end up 1 Byte off from the correct position
    sub rsi, 1
.copy:
    mov ecx, edx
    rep movsb                                   ; Repeat Move-Byte, copies the data from the memory address of RSI to the memory address of RDI
    cld
    ret


memcmp:                                         ; arguments are stored in RDI (void* ptr1), RSI (void* ptr2), RDX (unsigned int size)
    cld
    xor eax, eax                                ; EAX will store the return value
    mov ecx, edx
    repe cmpsb                                  ; Repeat-while-Equal Compare-Byte, compares memory and sets rflags accordingly: if they're equal, and ECX is not 0, it will repeat the process
    setnz al                                    ; Set-if-Non-Zero, sets AL is set to 1 if the zero flag is cleared, which means that values hold by the two memory locations are not equal
    ret
