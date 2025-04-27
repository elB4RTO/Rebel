.global memset
.global memcpy
.global memcmp
.global safe_copy


memset:                                         // arguments are stored in RDI (dst:u64), RSI (val:u8), RDX (size:u64)
    cld                                         // clear the direction flag to copy data from low memory address to high memory address
    mov rcx, rdx
    mov al, sil
    rep stosb                                   // Repeat Store-Byte, copies the value in AL to the location addressed by RDI
    ret


memcpy:                                         // arguments are stored in RDI (dst:u64), RSI (src:u64), RDX (size:u64)
    cld
    mov rcx, rdx
    rep movsb                                   // Repeat Move-Byte, copies the data from the memory address of RSI to the memory address of RDI
    ret


memcmp:                                         // arguments are stored in RDI (addr1:u64), RSI (addr2:u64), RDX (size:u64)
    cld
    xor rax, rax                                // RAX will store the return value
    mov rcx, rdx
    repe cmpsb                                  // Repeat-while-Equal Compare-Byte, compares memory and sets rflags accordingly: if they're equal and ECX is not 0, repeat the process
    setnz al                                    // Set-if-Non-Zero, AL is set to 1 if the zero flag is cleared, which means that values hold by the two memory locations are not equal
    ret


safe_copy:                                      // arguments are stored in RDI (dst:u64), RSI (src:u64), RDX (size:u64)
    cld
    cmp rsi, rdi                                // compare the two locations to avoid overwriting memory in case the two locations overlap: if destination is greater than source, copy backward, otherwise copy forward
    jae .copy
    mov rbx, rsi                                // copy the src pointer
    add rbx, rdx                                // add the size
    cmp rbx, rdi                                // compare with the dst pointer
    jbe .copy                                   // check if the address is less or equal to the dst address
.overlap:
    std                                         // set the direction flag to copy the data from high memory address to low memory address
    add rdi, rdx                                // data will be copied backward, so increment by the size to start from the end
    sub rdi, 1                                  // decrement by 1 to not start 1 Byte off from the correct position
    add rsi, rdx
    sub rsi, 1
.copy:
    mov rcx, rdx
    rep movsb                                   // Repeat Move-Byte, copies the data from the memory address of RSI to the memory address of RDI
    cld
    ret
