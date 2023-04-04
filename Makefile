ROOT := $(shell git rev-parse --show-toplevel)

KERN := kernel
TARGET_DIR := target/aarch64-unknown-none/release
TARGET := $(TARGET_DIR)/$(KERN)
BINARY := $(TARGET).bin
USER_DIRECTORY := $(TARGET_DIR)/user
SDCARD ?= $(ROOT)/user/fs.img
USER_PROGRAMS := cat echo fib heap init shell stack test
USER_DEBUG ?= init

QEMU := qemu-system-aarch64
QEMU_ARGS := -nographic -M raspi3b -serial null -serial mon:stdio \
			 -drive file=$(SDCARD),format=raw,if=sd -kernel

.PHONY: all build qemu transmit objdump nm check clean install test user image

all: build

build:
	@echo "+ Building build/$(KERN).elf [build/$@]"
	@cargo build --bin kernel --release --target aarch64-unknown-none

	@echo "+ Building build/$(KERN).bin [objcopy]"
	@objcopy -O binary $(TARGET) $(BINARY)

check:
	@cargo check

clippy:
	@cargo clippy

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
	@rm -rf $(USER_DIRECTORY)
	@mkdir $(USER_DIRECTORY)
	@echo "+ Building user programs"
	for program in $(USER_PROGRAMS) ; do\
		cargo build --bin $$program --target aarch64-unknown-none --release &&\
		objcopy -O binary $(TARGET_DIR)/$$program $(USER_DIRECTORY)/$$program;\
    done

image: user
	rm -f $(SDCARD)
	cargo run --target aarch64-apple-darwin --package image --bin image $(SDCARD) create
	ls $(USER_DIRECTORY)
	cargo run --target aarch64-apple-darwin --package image --bin image $(SDCARD) format fat32 0 $(USER_DIRECTORY)
	qemu-img resize $(SDCARD) 128M

user-objdump:
	objdump -dS $(TARGET_DIR)/$(USER_DEBUG)