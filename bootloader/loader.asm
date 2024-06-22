extern load_kernel

%define CODE_SEG 0x08
%define DATA_SEG 0x10

[BITS 16]

start:
    mov [DriveId], dl

cpu_features:
.test_cpuid:                                    ; check if CPUID is supported by attempting to flip the ID bit (bit 21) in the FLAGS register
    pushfd                                      ; copy FLAGS in to EAX via stack
    pop eax
    mov ecx, eax                                ; backup to ECX as well for comparing later on

    xor eax, (1<<21)                            ; flip the ID bit

    push eax                                    ; copy EAX to FLAGS via stack
    popfd

    pushfd                                      ; copy FLAGS back to EAX (with the flipped bit if CPUID is supported)
    pop eax

    push ecx                                    ; restore the old version of the FLAGS stored in ECX
    popfd

    xor eax, ecx                                ; if equal, the ID bit wasn't flipped and CPUID is not supported
    jz err_cpuid

.test_cpuid_extended_functions:                ; check if the extended functions of CPUID are available
    mov eax, 0x80000000
    cpuid
    cmp eax, 0x80000001
    jb err_cpuid

.test_cpuid_long_mode:
    mov eax, 0x80000001                         ; 0x80000001 makes CPUID return processor features inside EDX (among which there is LongMode and 1GB Page support)
    cpuid
    test edx, (1<<29)                           ; if bit 29 is true, LongMode is supported
    jz err_long_mode

.test_cpuid_paging:
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

.memory_info:                                   ; find a memory region big enough to load the kernel into
    cmp dword [es:di+16], 1                     ; check the memory type to be of type 1 (free memory)
    jne .memory_next
    cmp dword [es:di+4], 0                      ; check the higher part of the memory region address to be zero
    jne .memory_next
    mov eax, [es:di]                            ; store the lower part of the memory region address
    cmp eax, 0x100000                           ; check for the address to be 0x100000 (1 MiB past the start of memory)
    ja .memory_next
    cmp dword [es:di+12], 0                     ; check the length of the region to be large enough
    jne .memory_found
    add eax, [es:di+8]                          ; otherwise, add the lower part of the length to the base address
    cmp eax, 0x6500000                          ; and then compare it with the address plus the size of the image (0x100000 + 0x6400000, 1 MiB + 128 MiB)
    jb .memory_next

.memory_found:
    mov byte [MemFound], 1

.memory_next:
    add edi, 20                                 ; adjust EDI to point to the next memory address (each memory block is 20 Bytes)
    inc dword [es:0]                            ; increments the counter for the number of structures
    test ebx, ebx                               ; check if this was the last region
    jz .memory_done

    mov eax, 0xE820
    mov edx, 0x534d4150
    mov ecx, 20
    int 15h
    jnc .memory_info

.memory_done:
    cmp byte [MemFound], 1
    jne err_no_memory

a20_line:                                       ; A20 Line should be enabled, but better to check
    mov ax, 0xFFFF
    mov es, ax

    mov word [ds:0x7C00], 0xA200
    cmp word [es:0x7C10], 0xA200
    jne a20_line_done                           ; if equal, there's a high chance that the value was truncated at bit 20, so test once more
    mov word [0x7C00], 0xB200
    cmp word [es:0x7C10], 0xB200
    je err_a20_line                             ; if still equal, A20 is really not enabled
a20_line_done:

protected_mode_tmp:
    cli                                         ; disable BIOS interrupts while switching

    xor ax, ax                                  ; clear segments registers
    mov ds, ax

    lgdt [GDT32Desc]                            ; load the GDT (Global Descriptors Table)

    mov eax, cr0                                ; CR0 controls the behavior of the processor
    or eax, 1                                   ; set CR0 to enter Protected Mode
    mov cr0, eax

load_filesystem:
    mov ax, DATA_SEG
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
    mov edi, 0x100000                           ; 0x100000 is the memory address where the kernel partition will be loaded (1 MiB from the start of memory)

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
    mov cx, 7936                                ; 62 sectors left, 512 B each, copy 4 B each time (512 * 62 / 4)
    mov esi, 0x60000
.copy_remaining_loop:
    mov eax, [fs:esi]
    mov [fs:edi], eax

    add esi, 4
    add edi, 4
    loop .copy_remaining_loop

protected_mode:                                 ; switch to Protected Mode
    cli

    lidt [IDT]                                  ; now load the IDT (Interrupt Descriptors Table)

    mov eax, cr0
    or al, 1
    mov cr0, eax

    jmp CODE_SEG:protected_mode_start         ; load the new code segment descriptor

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

MsgCpuidErr:        db "CPUID not supported"
MsgCpuidErrLen:     equ $-MsgCpuidErr

MsgLongModeErr:     db "Long Mode not supported"
MsgLongModeErrLen:  equ $-MsgLongModeErr

MsgPagingErr:       db "1GB Pages not supported"
MsgPagingErrLen:    equ $-MsgPagingErr

MsgMemMapErr:       db "Failed to get Memory Map"
MsgMemMapErrLen:    equ $-MsgMemMapErr

MsgNoMemErr:        db "No suitable memory region"
MsgNoMemErrLen:     equ $-MsgNoMemErr

MsgA20LineErr:      db "A20 Line not enabled"
MsgA20LineErrLen:   equ $-MsgA20LineErr

MsgReadDiskErr:     db "Failed to read disk"
MsgReadDiskErrLen:  equ $-MsgReadDiskErr

MsgReadingDisk:     db "Reading disk ..."
MsgReadingDiskLen:  equ $-MsgReadingDisk

DriveId: db 0
MemFound: db 0
Buffer: times 16 db 0

[BITS 32]

%define PWS  00000111b                          ; Present | Writable | Supervisor
%define HUGE 10000000b                          ; Page Size

protected_mode_start:
    mov ax, DATA_SEG
    mov ss, ax
    mov ds, ax
    mov es, ax

paging:                                         ; finds a free memory area and initializes the paging structure which is used to translate from virtual address to pysical address (we will use 1 GB pages)
    cld
    mov edi, 0x70000                            ; addresses from 0x80000 to 0x90000 might be used by the BIOS, so use addresses between 0x70000 and 0x80000
    push edi                                    ; backup EDI since it will be modified by STOSB
    xor eax, eax                                ; clear EAX since its value will be used by STOSD
    mov ecx, 4096                               ; clear all the 4096 entries
    rep stosd                                   ; copy the value stored in EAX into the address pointed to by EDI

.identity_map:
    mov dword [0x70000], 0x71000|PWS
    mov dword [0x71000], 0x00000|PWS|HUGE             ; use 1 GiB page to identity map the first GiB

.map_kernel_pages:
    mov dword [0x70FF8], 0x72000|PWS                  ; 0x70FF8 = 0x70000 + 0x1FF*8
    mov dword [0x72FF0], 0x73000|PWS                  ; 0x72FF0 = 0x72000 + 0x1FE*8
    mov dword [0x73000], 0x6600000|PWS|HUGE           ; 0x73000 = 0x73000 + 0x000*8 (0x6600000 is the first 2MiB aligned address after the in-memory kernel partition)

.map_kernel_1to1:
    mov dword [0x72FF8], 0x00000|PWS|HUGE             ; 0x72FF8 = 0x72000 + 0x1FF*8

.map_kernel_reursive:
    mov dword [0x72FE8], 0x72000|PWS                  ; map the third-last entry to the first
    mov eax, 0x72000
    mov ebx, 0x6800000                          ; next 2 MiB page after 0x6600000
    mov ecx, 509
.kpm_loop:                                      ; map all 509 entries (0x72000 ~ 0x72FE0)
    push ebx
    or ebx, PWS|HUGE
    mov [eax], ebx
    pop ebx
    add eax, 0x8
    add ebx, 0x200000
    loop .kpm_loop

.enable_pae_pge:
    mov eax, cr4
    or eax, 0xA0                                ; set the bits for PAE (Page Address Extension) and PGE (Paging Global Extensions)
    mov cr4, eax                                ; set CR4 to activate 64-bit mode

.enable_long_mode:
    mov ecx, 0xC0000080                         ; set the LME (Long Mode Enable) bit in EFER MSR
    rdmsr                                       ; read from the model-specific register. Read MSR returns its value in EAX
    or eax, 0x100                               ; set the LM (Long Mode) bit
    wrmsr                                       ; copy the value back with Write MSR

.set_pml4t_address:
    mov eax, 0x70000                            ; set the address of the PML4T entry
    mov cr3, eax

.enable_paging:
    mov eax, cr0
    or eax, 0x80000000                          ; set the Page bit
    mov cr0, eax                                ; set CR0 to enable paging

long_mode:
    lgdt [GDT64Desc]                            ; load the GDT for Long Mode

    jmp CODE_SEG:long_mode_start                ; load the new code segment descriptor

end_32:
    hlt
    jmp short end_32

GDT32:
GDT32Null:
    dq 0x0000000000000000                       ; null descriptor
GDT32Code:                                      ; the offset is 0x08 (CS)
    dw 0xFFFF                                   ;  0~15 > limit (4GiB)
    dw 0x0000                                   ; 16~31 > base
    db 0x00                                     ; 32~39 > base
    db 0x9A                                     ; 40~47 > access (10011010)
    db 0xCF                                     ; 48~55 > flags (1100) + limit (xF)
    db 0x00                                     ; 56~63 > base
GDT32Data:                                      ; the offset 0x10 (DS, SS, ES, FS, GS)
    dw 0xFFFF                                   ;  0~15 > limit (4GiB)
    dw 0x0000                                   ; 16~31 > base
    db 0x00                                     ; 32~39 > base
    db 0x92                                     ; 40~47 > access (10010010)
    db 0xCF                                     ; 48~55 > flags (1100) + limit (xF)
    db 0x00                                     ; 56~63 > base
GDT32End:

    dw 0x0000                                   ; padding to align the GDT Descriptor address on a 4 Byte boundary
GDT32Desc:
    dw GDT32End - GDT32 - 1                     ; size of GDT
    dd GDT32                                    ; offset of GDT

[BITS 64]

long_mode_start:
    mov ax, DATA_SEG
    mov ss, ax
    mov ds, ax
    mov es, ax

launch_kernel:
    call load_kernel

    mov rsp, 0xFFFFFFFF80000000                 ; adjust the kernel stack pointer
    mov rax, 0xFFFFFFFF80000000                 ; jump to the kernel
    jmp rax

end_64:
    hlt
    jmp end_64

IDT:
    dw 0x0000                                   ; an invalid IDT pointing at 0, redirect interrupts while switching to long mode (nonetheless, non-maskable interrupts will still occurs, but they are generated for non-recoverable hardware errors)
    dd 0x00000000

GDT64:
GDT64Null:
    dq 0x0000000000000000
GDT64Code:
    dw 0x0000                                   ;  0~15 > limit (unused)
    dw 0x0000                                   ; 16~31 > base  0~15 (unused)
    db 0x00                                     ; 32~39 > base 16~23 (unused)
    db 0x9A                                     ; 40~47 > access (10011010)
    db 0xA0                                     ; 48~55 > flags (1010) + limit (unused)
    db 0x00                                     ; 56~63 > base 24~31 (unused)
GDT64Data:
    dw 0x0000                                   ;  0~15 > limit (unused)
    dw 0x0000                                   ; 16~31 > base  0~15 (unused)
    db 0x00                                     ; 32~39 > base 16~23 (unused)
    db 0x92                                     ; 40~47 > access (10010010)
    db 0xC0                                     ; 48~55 > flags (1100) + limit (unused)
    db 0x00                                     ; 56~63 > base 24~31 (unused)
GDT64End:

    dw 0x0000                                   ; padding to align the GDT Descriptor address on a 4 Byte boundary
GDT64Desc:
    dw GDT64End - GDT64 - 1
    dd GDT64

