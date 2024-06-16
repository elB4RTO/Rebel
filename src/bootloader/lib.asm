
section .text

global memset
global memmove
global memcpy
global memcmp


memset:             ; arguments are passed to registers RDI (void* buffer), RSI (char value), RDX (int size)
    cld                 ; clears the direction flag, so that it copies data from low memory address to high memory address
    mov ecx,edx         ; EDX stores the size, so we move it to ECX
    mov al,sil
    rep stosb           ; [repeate store-byte] copies the value in AL to the memory addressed by RDI
    ret


memmove:            ; we define them to do the same thing
memcpy:             ; arguments are passed to registers RDI (void* dst), RSI (void* src), RDX (int size)
    cld
    cmp rsi,rdi         ; we compare the two locations to avoid overwriting memory in case the two locations overlap. so if destination is greater than source, we copy backward, otherwise we copy forward
    jae .copy
    mov r8,rsi          ; we copy the src pointer
    add r8,rdx          ; we add the size
    cmp r8,rdi          ; and compare with the dst pointer
    jbe .copy           ; if the address is less or equal to the dst address, we jump to the copy part
.overlap:
    std                 ; sets the direction flag, which will copy the data from high memory address to low memory address
    add rdi,rdx         ; we adjust the addresses by incrementing by the size, so that we can copy the data backward
    add rsi,rdx
    sub rdi,1           ; than we decrement them by 1, or we will end 1 Byte off from the correct position
    sub rsi,1
.copy:
    mov ecx,edx
    rep movsb           ; [repeat move-byte] copies the data from the memory address of RSI to the memory address of RDI
    cld
    ret


memcmp:             ; arguments are passed to registers RDI (void* src1), RSI (void* src2), RDX (int size)
    cld
    xor eax,eax         ; we clear the EAX register, which will store the return value
    mov ecx,edx         ; EDX stores the size, so we move it to ECX
    repe cmpsb          ; [repeate-while-equal compare-byte] compares memory and sets rflags accordingly: if they're equal, and ECX is not 0, it will repeat the process
    setnz al            ; [set-if-non-zero] if the zero flag is cleared, AL is set to 1, which means that values hold by the two memory locations are not equal, otherwise if EAX is 0 it means that they're equal
    ret
