arch ?= x86_64
target ?= $(arch)-blog_os
rust_os := target/$(target)/debug/libblog_os.a
kernel := build/kernel-$(arch).bin
iso := build/os-$(arch).iso

linker_script := bootloader/arch/$(arch)/linker.ld
grub_cfg := bootloader/arch/$(arch)/grub.cfg
assembly_source_files := $(wildcard bootloader/arch/$(arch)/*.asm)
assembly_object_files := $(patsubst bootloader/arch/$(arch)/%.asm, \
	build/arch/$(arch)/%.o, $(assembly_source_files))

.PHONY: all clean run iso kernel

all: $(kernel)

clean:
	@rm -r build

run: $(iso)
	@qemu-system-x86_64 -cdrom $(iso)

test: $(iso)
	@qemu-system-x86_64 -cdrom $(iso) -device isa-debug-exit,iobase=0xf4,iosize=0x04 -serial stdio

iso: $(iso)

$(iso): $(kernel) $(grub_cfg)
	@mkdir -p build/isofiles/boot/grub
	@cp $(kernel) build/isofiles/boot/kernel.bin
	@cp $(grub_cfg) build/isofiles/boot/grub
	@grub-mkrescue -o $(iso) build/isofiles 2> /dev/null
	@rm -r build/isofiles

$(kernel): kernel $(rust_os) $(assembly_object_files) $(linker_script)
	@x86_64-elf-ld -n --gc-sections -T $(linker_script) -o $(kernel) $(assembly_object_files) $(rust_os)

kernel:
	@RUST_TARGET_PATH=$(shell pwd) cargo build

build/arch/$(arch)/%.o: bootloader/arch/$(arch)/%.asm
	@mkdir -p $(shell dirname $@)
	@nasm -felf64 $< -o $@
