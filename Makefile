ROOT := $(shell git rev-parse --show-toplevel)

KERN := kernel
TARGET_DIR := target/aarch64-unknown-none/release/
TARGET := $(TARGET_DIR)/$(KERN)
BINARY := $(TARGET).bin
SDCARD ?= $(ROOT)/user/fs.img
USER_PROGRAMS := cat echo fib heap init shell stack

QEMU := qemu-system-aarch64
QEMU_ARGS := -nographic -M raspi3b -serial null -serial mon:stdio \
			 -drive file=$(SDCARD),format=raw,if=sd -kernel

.PHONY: all build qemu transmit objdump nm check clean install test user image

all: build

build:
	@echo "+ Building build/$(KERN).elf [build/$@]"
	@cargo build --bin kernel --release

	@echo "+ Building build/$(KERN).bin [objcopy]"
	@llvm-objcopy -O binary $(TARGET) $(BINARY)

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
	@echo "+ Building user programs"
	@for program in $(USER_PROGRAMS) ; do 											\
		cargo build --bin $$program --release;											\
		llvm-objcopy -O binary $(TARGET_DIR)/$$program $(TARGET_DIR)/$$program.bin;		\
    done

image:
	cd user; ./build.sh
