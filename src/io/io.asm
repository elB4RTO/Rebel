.section .text

.global in_byte
.global in_word
.global out_byte
.global out_word


in_byte:
    mov rdx, rdi                                // RDI holds the I/O port number
    in al, dx                                   // read 1 Byte from the I/O port
    ret

in_word:
    mov rdx, rdi                                // RDI holds the I/O port number
    in ax, dx                                   // read 1 Word from the I/O port
    ret

out_byte:
    mov rdx, rdi                                // RDI holds the I/O port number
    mov rax, rsi                                // RSI holds the value
    out dx, al                                  // write 1 Byte into the I/O port
    ret

out_word:
    mov rdx, rdi                                // RDI holds the I/O port number
    mov rax, rsi                                // RSI holds the value
    out dx, ax                                  // write 1 Word into the I/O port
    ret
