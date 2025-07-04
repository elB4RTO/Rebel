
VM = qemu

ifeq ($(VM),bochs)
EMULATOR = bochs -q
else
EMULATOR = qemu-system-x86_64 -hda disk.img -m 2G -cpu qemu64,pdpe1gb
endif

CFLAGS = -nostdlib -nostartfiles -nodefaultlibs -fno-builtin -ffreestanding -fno-stack-protector -fomit-frame-pointer -falign-jumps -falign-functions -falign-labels -falign-loops -mno-red-zone -Wall -Werror -Wno-unused-function -Wno-unused-label -Wno-unused-parameter -Wno-cpp

CARGO_FLAGS = --offline
CARGO_TARGET_DIR = build/kernel/x86_64-unknown-none/debug

DEBUG = 0

ifeq ($(DEBUG), 0)
CFLAGS += -Os -finline-functions
CARGO_FLAGS += --release
CARGO_TARGET_DIR = build/kernel/x86_64-unknown-none/release
else
CFLAGS += -O0 -g
endif

TEST = 0
TESTS_DEBUG = 0

ifeq ($(TEST), 1)
CARGO_FLAGS += --features unit_tests
endif

HUGE_STACK = 0

ifeq ($(HUGE_STACK), 1)
CARGO_FLAGS += --features huge_stack
endif

MOUNT_DIR = build/mnt

BOOTLOADER = build/bootloader.bin

KERNEL = build/kernel.bin


#### PHONY TARGETS ####

.PHONY: all
all: clean build create run

.PHONY: redo
redo: clean_kernel build create run

.PHONY: clean
clean:
	test -e build && rm -rf build || return 0
	test -e disk.img && rm -rf disk.img || return 0

.PHONY: clean_kernel
clean_kernel:
	test -e build/kernel.elf && rm -f build/kernel.elf || return 0
	test -e $(CARGO_TARGET_DIR)/librebel.a && rm -f $(CARGO_TARGET_DIR)/librebel.a || return 0
	test -e build/kernel.o && rm -f build/kernel.o || return 0
	test -e build/kernel.bin && rm -f build/kernel.bin || return 0
	test -e build/partitions/kernel.img && rm -f build/partitions/kernel.img || return 0

.PHONY: prepare
prepare:
	mkdir -p build
	mkdir -p build/bootloader
	mkdir -p build/partitions
	mkdir -p $(MOUNT_DIR)

.PHONY: build
build: prepare bootloader kernel

.PHONY: bootloader
bootloader: $(BOOTLOADER)

.PHONY: kernel
kernel: $(KERNEL)

.PHONY: create
create: disk.img build/partitions/boot.img build/partitions/kernel.img build/bootloader.bin
	dd if=build/partitions/boot.img of=disk.img bs=512 count=62 conv=notrunc
	dd if=build/partitions/kernel.img of=disk.img bs=512 seek=63 conv=notrunc

.PHONY: run
run:
	$(EMULATOR)


#### CREATE TARGETS ####


disk.img:
	touch $@

build/partitions/boot.img: build/bootloader.bin
	dd if=$^ of=$@ bs=512 count=62 conv=notrunc

build/partitions/kernel.img: build/kernel.bin
	dd if=/dev/zero of=$@ bs=1M count=100
	mkfs.fat -F 16 -f 1 -R 1 -D 0x80 -n REBELKERNEL $@
	sudo mount -t vfat $@ $(MOUNT_DIR)
	sudo cp $^ $(MOUNT_DIR) && sleep 1
	sudo umount $(MOUNT_DIR)


#### BUILD TARGETS ####


build/bootloader/boot.bin: bootloader/boot.asm
	nasm -f bin $^ -o $@

build/bootloader/loader.elf: bootloader/loader.asm
	nasm -f elf64 $^ -o $@

build/bootloader/entry.o: bootloader/entry.c
	gcc -std=c99 -mcmodel=large $(CFLAGS) -c $^ -o $@

build/bootloader/lib.elf: bootloader/lib.asm
	nasm -f elf64 $^ -o $@

build/bootloader/print.o: bootloader/print.c
	gcc -std=c99 -mcmodel=large $(CFLAGS) -c $^ -o $@

build/bootloader/file.o: bootloader/file.c
	gcc -std=c99 -mcmodel=large $(CFLAGS) -c $^ -o $@

build/bootloader/loader.o: build/bootloader/loader.elf build/bootloader/entry.o build/bootloader/file.o build/bootloader/print.o build/bootloader/lib.elf
	ld -nostdlib -T bootloader/link.ld $^ -o $@

build/bootloader/loader.bin: build/bootloader/loader.o
	objcopy -O binary $^ $@

build/bootloader.bin: build/bootloader/boot.bin build/bootloader/loader.bin
	dd if=build/bootloader/boot.bin >> $@
	dd if=build/bootloader/loader.bin >> $@


build/kernel.elf: src/kernel.asm
	nasm -f elf64 $^ -o $@

$(CARGO_TARGET_DIR)/librebel.a:
	cargo build $(CARGO_FLAGS)

build/kernel.o: build/kernel.elf $(CARGO_TARGET_DIR)/librebel.a
	ld -T src/link.ld $^ -o $@

build/kernel.bin: build/kernel.o
	objcopy -O binary $^ $@
