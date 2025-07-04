.section .text

.global in_byte
.global in_word
.global out_byte
.global out_word


in_byte:
    mov dx, di                                  // RDI holds the I/O port number
    in al, dx                                   // read 1 Byte from the I/O port
    ret

in_word:
    mov dx, di                                  // RDI holds the I/O port number
    in ax, dx                                   // read 1 Word from the I/O port
    ret

in_double_word:
    mov dx, di                                  // RDI holds the I/O port number
    in eax, dx                                  // read 1 DoubleWord from the I/O port
    ret

out_byte:
    mov dx, di                                  // RDI holds the I/O port number
    mov ax, si                                  // RSI holds the value
    out dx, al                                  // write 1 Byte into the I/O port
    ret

out_word:
    mov dx, di                                  // RDI holds the I/O port number
    mov ax, si                                  // RSI holds the value
    out dx, ax                                  // write 1 Word into the I/O port
    ret

out_double_word:
    mov dx, di                                  // RDI holds the I/O port number
    mov eax, esi                                // RSI holds the value
    out dx, eax                                 // write 1 Word into the I/O port
    ret
