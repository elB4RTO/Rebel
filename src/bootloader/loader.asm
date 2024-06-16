[ORG 0x7E00]

[BITS 16]

start:
    mov [DriveId], dl

cpu_features:
    mov eax, 0x80000000
    cpuid
    cmp eax, 0x80000001
    jb err_cpuid

    mov eax, 0x80000001                         ; 0x80000001 makes CPUID return processor features inside EDX (among which there is LongMode and 1GB Page support)
    cpuid
    test edx, (1<<29)                           ; if bit 29 is true, LongMode is supported
    jz err_long_mode
    test edx, (1<<26)                           ; if bit 26 is true, 1 GB Pages is supported
    jz err_paging

    mov ax, 0x2000                              ; address of the memory info block, actually 0x20000: 0x2000 (segment) * 16 + 0x0 (offset)
    mov es, ax

system_memory_map:
    mov eax, 0xE820
    mov edx, 0x534d4150                         ; 'SMAP'
    mov ecx, 20                                 ; size of the result buffer (in Bytes)
    mov dword [es:0], 0
    mov edi, 8                                  ; offset for the address where the result is saved, which becomes 0x20008: 0x20000 (segment) + 8 (offset)
    xor ebx, ebx                                ; must be cleared
    int 15h                                     ; System Memory Map (www.ctyme.com/intr/rb-1741.htm)
    jc err_memory_map

memory_info:                                    ; find a memory region big enough to load the kernel into
    cmp dword [es:di+16], 1                     ; check the memory type to be of type 1 (free memory)
    jne memory_next
    cmp dword [es:di+4], 0                      ; check the higher part of the memory region address to be zero
    jne memory_next
    mov eax, [es:di]                            ; store the lower part of the memory region address
    cmp eax, 0x30000000                         ; is it the address we are looking for?
    ja memory_next
    cmp dword [es:di+12], 0                     ; check the length of the region to be large enough
    jne memory_found
    add eax, [es:di+8]                          ; otherwise, add the lower part of the length to the base address
    cmp eax, 0x30000000 + 100*1024*1024         ; and then compare it with the address plus the size of the image (100 MB)
    jb memory_next

memory_found:
    mov byte [LoadImage], 1

memory_next:
    add edi, 20                                 ; adjust EDI to point to the next memory address (each memory block is 20 Bytes)
    inc dword [es:0]                            ; increments our counter for the number of structures
    test ebx, ebx                               ; check if this was the last region
    jz memory_done

    mov eax, 0xE820
    mov edx, 0x534d4150
    mov ecx, 20
    int 15h
    jnc memory_info

memory_done:
    cmp byte [LoadImage], 1
    jne err_no_memory

a20_line:                                       ; A20 Line should be enabled, but better to check
    mov ax, 0xFFFF
    mov es, ax

    mov word [ds:0x7c00], 0xA200
    cmp word [es:0x7c10], 0xA200
    jne a20_line_done                           ; if equal, there's a high chance that the value was truncated at bit 20, so test once more
    mov word [0x7c00], 0xB200
    cmp word [es:0x7c10], 0xB200
    je err_a20_line                             ; if still equal, A20 is really not enabled

a20_line_done:
    xor ax, ax
    mov es, ax

protected_mode_tmp:
    cli                                         ; disable BIOS interrupts while switching
    lgdt [GDT32Ptr]                             ; load the GDT (Global Descriptors Table)

    mov eax, cr0                                ; CR0 controls the behavior of the processor
    or eax, 1                                   ; set CR0 to enter Protected Mode
    mov cr0, eax

load_filesystem:
    mov ax, 0x10
    mov fs, ax                                  ; load the filesystem with the segment selector 16 (see GDT32Data)

big_real_mode:
    mov eax, cr0
    and al, 0xFE                                ; set CR0 to switch back to Real Mode, which is now Big Real Mode
    mov cr0, eax
    sti

read_fat16:
    push di
    call msg_reading
    pop di

    mov cx, 2048                                ; read 204862 total sectors (204800 + 62 <=> 100 MiB + 31 KiB <=> kernel + bootloader), 100 sectors at a time (204862 / 100)
    xor ebx, ebx
    mov edi, 0x1000000                          ; 0x1000000 is the memory address where the kernel partition will be loaded (16 MiB from the start of memory)

    xor ax, ax
    mov fs, ax                                  ; clear FS to avoid issues while reading the FAT16 image

read_data:
    push ecx
    push ebx
    push edi
    push fs

    mov ax, 100                                 ; read 100 sectors
    call read_sectors
    test al, al                                 ; after reading sectors, AL stores the carry flag
    jnz err_read_disk

    pop fs
    pop edi
    pop ebx                                     ; ECX popped later

copy_data:
    mov cx, 12800                               ; 100 sectors, 512 B each, copy 4 B each time (512 * 100 / 4)
    mov esi, 0x60000                            ; 0x60000 is the address where the read data is loaded
.copy_loop:
    mov eax, [fs:esi]                           ; copy the data into EAX (use FS to make sure the address will be inside limits)
    mov [fs:edi], eax                           ; move the data to the high memory address stored EDI

    add esi, 4                                  ; increment ESI and ADI by 4 B to move to the next position
    add edi, 4
    loop .copy_loop                             ; loop until CX is empty

    pop ecx

    add ebx, 100                                ; we adjust EBX to read the next sectors
    loop read_data

read_remaining_sectors:                         ; after reading is done, there may be the last sectors to read (less than 100)
    push edi
    push fs

    mov ax, 62                                  ; the number of sectors left to read (204862 % 100)
    call read_sectors
    test al, al
    jnz err_read_disk

    pop fs
    pop edi

copy_remaining_data:
    mov cx, (((203*16*63) % 100) * 512)/4       ; get the size left to copy
    mov esi, 0x60000
.copy_remaining_loop:
    mov eax, [fs:esi]
    mov [fs:edi], eax

    add esi, 4
    add edi, 4
    loop .copy_remaining_loop

protected_mode:                                 ; switch to Protected Mode
    cli
    lidt [IDT32Ptr]                             ; now load the IDT (Interrupt Descriptors Table)

    mov eax, cr0
    or eax, 1
    mov cr0, eax

    jmp 8:protected_mode_start                  ; the code segment descriptor is the second entry, so it starts at 8 Bytes, with an offset defined by protected_mode_start

read_sectors:
    mov si, Buffer
    mov word [si], 0x10                         ; the size of the buffed (16 Bytes)
    mov word [si+2], ax                         ; the number of sectors to read
    mov word [si+4], 0                          ; the offset of the memory address
    mov word [si+6], 0x6000                     ; the segment of the memory address (0x6000 * 16 + 0 = 0x60000)
    mov dword [si+8], ebx                       ; address lo of the logical block address. represents the sector of the disk (starting from 0)
    mov dword [si+12], 0                        ; address hi of the logical block address
    mov dl, [DriveId]
    mov ah, 0x42                                ; Extended Read (https://www.ctyme.com/intr/rb-0708.htm)
    int 13h

    setc al
    ret

err_cpuid:
    mov bp, MsgCpuidErr
    mov cx, MsgCpuidErrLen
    call print_bios
    jmp end_16

err_long_mode:
    mov bp, MsgLongModeErr
    mov cx, MsgLongModeErrLen
    call print_bios
    jmp end_16

err_paging:
    mov bp, MsgPagingErr
    mov cx, MsgPagingErrLen
    call print_bios
    jmp end_16

err_memory_map:
    mov si, MsgMemMapErr
    mov cx, MsgMemMapErrLen
    mov bx, 0xC
    call print_text
    jmp end_16

err_no_memory:
    mov si, MsgNoMemErr
    mov cx, MsgNoMemErrLen
    mov bx, 0xC
    call print_text
    jmp end_16

err_a20_line:
    mov si, MsgA20LineErr
    mov cx, MsgA20LineErrLen
    mov bx, 0xC
    call print_text
    jmp end_16

err_read_disk:
    mov si, MsgReadDiskErr
    mov cx, MsgReadDiskErrLen
    mov bx, 0xC
    call print_text
    jmp end_16

msg_reading:
    mov si, MsgReadingDisk
    mov cx, MsgReadingDiskLen
    mov bx, 0xF
    call print_text
    ret

print_bios:
    mov ah, 0x13
    mov al, 1
    mov bx, 0xC
    xor dx, dx
    int 10h
    ret

print_text:
    mov ax, 0xB800
    mov es, ax
    xor di, di
.print_loop:
    mov al, [si]
    mov [es:di], al
    mov byte [es:di+1], bl
    add di, 2
    add si, 1
    loop .print_loop
    ret

end_16:
    hlt
    jmp short end_16

MsgCpuidErr:    db "CPUID not supported"
MsgCpuidErrLen: equ $-MsgCpuidErr

MsgLongModeErr:    db "Long Mode not supported"
MsgLongModeErrLen: equ $-MsgLongModeErr

MsgPagingErr:    db "1GB Pages not supported"
MsgPagingErrLen: equ $-MsgPagingErr

MsgMemMapErr:    db "Failed to get Memory Map"
MsgMemMapErrLen: equ $-MsgMemMapErr

MsgNoMemErr:    db "No suitable memory region"
MsgNoMemErrLen: equ $-MsgNoMemErr

MsgA20LineErr:    db "A20 Line not enabled"
MsgA20LineErrLen: equ $-MsgA20LineErr

MsgReadDiskErr:    db "Failed to read disk"
MsgReadDiskErrLen: equ $-MsgReadDiskErr

MsgReadingDisk:    db "Reading disk ..."
MsgReadingDiskLen: equ $-MsgReadingDisk

[BITS 32]

protected_mode_start:
    mov ax, 0x10                                ; the data segment descriptor is the third entry, so it starts at 16 Bytes
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov esp, 0x7c00

paging:                                         ; finds a free memory area and initializes the paging structure which is used to translate from virtual address to pysical address (we will use 1 GB pages)
    cld
    mov edi, 0x70000                            ; addresses from 0x80000 to 0x90000 might be used by the BIOS, so use addresses between 0x70000 and 0x80000
    xor eax, eax
    mov ecx, 0x10000/4
    rep stosd
    mov dword [0x70000], 0x71007                ; only set up the first entry (1GB) of the page map level 4 (which can represent 512 GB). set it to be readable and writable, and only accessible by the kernel
    mov dword [0x71000], 10000111b              ; set up the first entry of the page directory pointer table

    mov eax, (0xFFFF800000000000>>39)           ; get the 9 bits page map level 4 value, which is located at bit 39 of the address
    and eax, 0x1FF                              ; we get the 9 bits index value
    mov dword [0x70000 + eax*8], 0x72003        ; use the index to locate the corresponding entry in the table (each entry is 8 Bytes)
    mov dword [0x72000], 10000011b              ; re-map the kernel to the virtual address without really copying the kernel elsewhere, so set the 1GB physical page to the same physical page where the kernel is located

long_mode:
    lgdt [GDT64Ptr]                             ; load the Long Mode GDT

    mov eax, cr4                                ; set CR4 to activate 64-bit mode
    or eax, (1<<5)                              ; the fifth bit is the PAE bit (Physical Address Extension)
    mov cr4, eax

    mov eax, 0x70000                            ; set the address of the page map
    mov cr3, eax

    mov ecx, 0xC0000080                         ; set the eight bit to 1 for the Extended Feature Enable
    rdmsr                                       ; Read MSR returns its value in EAX
    or eax, (1<<8)
    wrmsr                                       ; copy the value back with Write MSR

    mov eax, cr0                                ; set the 31st bit of CR0 to 1 to enable paging
    or eax, (1<<31)
    mov cr0, eax

    jmp 8:long_mode_start                       ; load the new code segment descriptor

end_32:
    hlt
    jmp short end_32

[BITS 64]

long_mode_start:
    mov rsp, 0x7C00

    cld                                         ; clear the direction flag so that MOV will process the data from low memory address to high memory address (in forward direction)
    mov rdi, 0x100000                           ; the destination address
    mov rsi, entry                              ; the source address
    mov rcx, 512*15/8                           ; 15 sectors, 512 Bytes each, copy 8 Bytes per time
    rep movsq

    mov rax, 0xFFFF800000100000                 ; the address of the kernel entry
    jmp rax

end_64:
    hlt
    jmp end_64

DriveId: db 0
LoadImage: db 0

Buffer: times 16 db 0

GDT32:
    dq 0
GDT32Code:
    dw 0xFFFF
    dw 0
    db 0
    db 0x9A
    db 0xCF
    db 0
GDT32Data:
    dw 0xFFFF
    dw 0
    db 0
    db 0x92
    db 0xCF
    db 0

GDT32Len: equ $-GDT32
GDT32Ptr: dw GDT32Len-1
          dd GDT32

IDT32Ptr: dw 0                                  ; an invalid IDT pointing at 0, redirect interrupts while switching to long mode (nonetheless, non-maskable interrupts will still occurs, but they are generated for non-recoverable hardware errors)
          dd 0

GDT64:
    dq 0
    dq 0x0020980000000000

GDT64Len: equ $-GDT64
GDT64Ptr: dw GDT64Len-1
          dd GDT64

entry:                                          ; compiled kernel entry will be appended here
