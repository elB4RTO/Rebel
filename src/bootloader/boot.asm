[ORG 0x7C00]

[BITS 16]

; emulate a FAT16 file system
jmp short start
nop

; BPB (Bios Parameter Block)
OEMIdentifier       db 'REBEL   '               ; must be exactly 8 Bytes
BytesPerSector      dw 0x0200                   ; 512
SectorsPerCluster   db 0x80
ReservedSectors     dw 0x003E                   ; 62
NumberOfFATs        db 0x02                     ; must be 2
RootEntriesCount    dw 0x0200
NumberOfSectors     dw 0x003E                   ; 62
MediaType           db 0xF8                     ; disk
SectorsPerFat       dw 0x0100
SectorsPerTrack     dw 0x0020
NumberOfHeads       dw 0x0040
HiddenSectors       dd 0x00000000
TotalSectors32      dd 0x00000000

; EBPB (Extended Bios Parameter Block)
DriveNumber         db 0x80                     ; fixed disk
Reserved            db 0x00                     ; Windows NT stuff
Signature           db 0x29
VolumeID            dd 0xD105
VolumeIDString      db 'REBEL BOOT '            ; must be exactly 11 Bytes
SystemIDString      db 'FAT16   '               ; must be exactly 8 Bytes

start:
    mov ax, 3                                   ; switch to video mode 3 (80x25)
    int 10h

    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7c00                              ; stack memory will go below this address

test_disk_extensions:
    mov [DriveId], dl
    mov ah, 0x41
    mov bx, 0x55AA
    int 13h                                     ; Disk Extensions (https://www.ctyme.com/intr/rb-0706.htm)
    jc err_disk_ext
    cmp bx, 0xAA55
    jne err_disk_ext

load_loader:                                    ; load the loader into memory and call it
    mov si, Buffer
    mov word [si], 16                           ; Bytes to read per time
    mov word [si+2], 62                         ; number of sectors to read
    mov word [si+4], 0x7E00                     ; offset of the memory address (here 0x7E00 is also the actual address)
    mov word [si+6], 0                          ; segment part of the address (here 0 so physical address is the same as logical address)
    mov dword [si+8], 1                         ; address lo of the logical block address
    mov dword [si+12], 0                        ; address hi of the logical block address
    mov dl, [DriveId]
    mov ah, 0x42
    int 13h                                     ; Extended Read (https://www.ctyme.com/intr/rb-0708.htm)
    jc err_read_disk

    mov dl, [DriveId]
    jmp 0x7E00                                  ; jump to the loader

err_disk_ext:
    mov bp, MsgDiskExtErr
    mov cx, MsgDiskExtErrLen
    jmp short print_err

err_read_disk:
    mov bp, MsgLoadLoaderErr
    mov cx, MsgLoadLoaderErrLen
    jmp short print_err

print_err:
    mov ah, 0x13
    mov al, 1
    mov bx, 0xC
    xor dx, dx
    int 10h

end:
    hlt
    jmp short end


DriveId: db 0

MsgDiskExtErr:    db "DiskExtensions not supported",0
MsgDiskExtErrLen: equ $-MsgDiskExtErr

MsgLoadLoaderErr:    db "Failed to load loader",0
MsgLoadLoaderErrLen: equ $-MsgLoadLoaderErr

Buffer: times 16 db 0

times 0x1FE-($-$$) db 0
;times (0x1BE-($-$$)) db 0

; FSI (File System Info) (FAT32 stuff)
;db 0x80                                         ; a bootable partition
;db 0x01, 0x01, 0x00                             ; the starting CHS (Cylinder, Head, Sector)
;db 0x06                                         ; partition type
;db 0x0F, 0x3F, 0xCA                             ; ending CHS
;dd 0x3F                                         ; LBA (Logical Block Address), the index of the starting sector
;dd 0x00031F11                                   ; number of sectors of the partition
;times 48 db 0                                   ; unused fields

dw 0xAA55                                       ; FAT signature
