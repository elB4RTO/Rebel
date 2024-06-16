.PHONY: all clean prepare build create run

DEBUG = 1

VM = qemu

CFLAGS = -nostdlib -nostartfiles -nodefaultlibs -fno-builtin -ffreestanding -fno-stack-protector -fomit-frame-pointer -falign-jumps -falign-functions -falign-labels -falign-loops -mno-red-zone -Wall -Werror -Wno-unused-function -Wno-unused-label -Wno-unused-parameter -Wno-cpp

ifeq ($(DEBUG), 0)
CARGO_BUILD_MODE = --release
CFLAGS += -O3 -finline-functions
else
CARGO_BUILD_MODE =
CFLAGS += -O0 -g
endif

MOUNT_DIR = build/mnt

BOOTLOADER = build/bootloader.bin

KERNEL = build/kernel.bin


#### PHONY TARGETS ####


all: clean build create run

clean:
	test -e build && rm -rf build || return 0
	test -e disk.img && rm -rf disk.img || return 0

prepare:
	mkdir -p build
	mkdir -p build/bootloader
	mkdir -p build/partitions
	mkdir -p $(MOUNT_DIR)

build: prepare $(BOOTLOADER) $(KERNEL)
	#cargo build --offline $(CARGO_BUILD_MODE)

create: disk.img build/partitions/boot.img  build/partitions/kernel.img build/bootloader.bin
	dd if=build/partitions/boot.img of=disk.img bs=512 count=62 conv=notrunc
	dd if=build/partitions/kernel.img of=disk.img bs=512 seek=63 conv=notrunc

run:
ifeq ($(VM),bochs)
	bochs -q
else
	qemu-system-x86_64 -hda disk.img -m 1G -cpu qemu64,pdpe1gb
endif


#### BUILD TARGETS ####

disk.img:
	touch $@

build/partitions/boot.img: build/bootloader.bin
	dd if=$^ of=$@ bs=512 count=62 conv=notrunc

build/partitions/kernel.img: build/kernel.bin
	dd if=/dev/zero of=$@ bs=1M count=100
	mkfs.fat -F 16 -f 1 -R 1 -D 0x80 -n REBELKERNEL $@
	sudo mount -t vfat $@ $(MOUNT_DIR)
	sudo cp $^ $(MOUNT_DIR)
	sudo umount $(MOUNT_DIR)


build/bootloader/boot.bin: src/bootloader/boot.asm
	nasm -f bin $^ -o $@

build/bootloader/loader.bin: src/bootloader/loader.asm
	nasm -f bin $^ -o $@

build/bootloader/entry.elf: src/bootloader/entry.asm
	nasm -f elf64 $^ -o $@

build/bootloader/entry.o: src/bootloader/entry.c
	gcc -std=c99 -mcmodel=large $(CFLAGS) -c $^ -o $@

build/bootloader/lib.elf: src/bootloader/lib.asm
	nasm -f elf64 $^ -o $@

build/bootloader/debug.o: src/bootloader/debug.c
	gcc -std=c99 -mcmodel=large $(CFLAGS) -c $^ -o $@

build/bootloader/print.o: src/bootloader/print.c
	gcc -std=c99 -mcmodel=large $(CFLAGS) -c $^ -o $@

build/bootloader/file.o: src/bootloader/file.c
	gcc -std=c99 -mcmodel=large $(CFLAGS) -c $^ -o $@

build/bootloader/launcher.o: build/bootloader/entry.elf build/bootloader/entry.o build/bootloader/file.o build/bootloader/print.o build/bootloader/debug.o build/bootloader/lib.elf
	ld -nostdlib -T src/bootloader/link.ld $^ -o $@

build/bootloader/launcher.bin: build/bootloader/launcher.o
	objcopy -O binary $^ $@

build/bootloader.bin: build/bootloader/boot.bin build/bootloader/loader.bin build/bootloader/launcher.bin
	dd if=build/bootloader/boot.bin >> $@
	dd if=build/bootloader/loader.bin >> $@
	dd if=build/bootloader/launcher.bin >> $@


build/kernel.elf: src/kernel.asm
	nasm -f elf64 $^ -o $@

build/kernel.o: src/kernel.c
	gcc -std=c99 -mcmodel=large $(CFLAGS) -c $^ -o $@

build/kernel: build/kernel.elf build/kernel.o  build/bootloader/print.o  build/bootloader/lib.elf
	ld -nostdlib -T src/link.ld $^ -o $@

build/kernel.bin: build/kernel
	objcopy -O binary $^ $@
