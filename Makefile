ROOT := $(shell git rev-parse --show-toplevel)

KERN := kernel
TARGET_DIR := target/aarch64-unknown-none/release/
TARGET := $(TARGET_DIR)/$(KERN)
BINARY := $(TARGET).bin
SDCARD ?= $(ROOT)/user/fs.img

QEMU := qemu-system-aarch64
QEMU_ARGS := -nographic -M raspi3b -serial null -serial mon:stdio \
			 -drive file=$(SDCARD),format=raw,if=sd -kernel

.PHONY: all build qemu transmit objdump nm check clean install test

all: build

build:
	@echo "+ Building build/$(KERN).elf [build/$@]"
	@cargo build --bin kernel --release

	@echo "+ Building build/$(KERN).bin [objcopy]"
	@llvm-objcopy -O binary $(TARGET) $(BINARY)

check:
	@cargo check

qemu: build
	$(QEMU) $(QEMU_ARGS) $(BINARY)

qemu-gdb: build
	$(QEMU) $(QEMU_ARGS) $(BINARY) -s -S

qemu-asm: build
	$(QEMU) $(QEMU_ARGS) $(BINARY) -d in_asm

objdump: build
	cargo objdump -- -disassemble -no-show-raw-insn -print-imm-hex $(TARGET)

clean:
	cargo clean

user:
	@echo "+ Building user programs"
	cargo build --bin fib --release
	llvm-objcopy -O binary $(TARGET_DIR)/fib $(TARGET_DIR)/fib.bin

drive:
