ROOT := $(shell git rev-parse --show-toplevel)

KERN := kernel
TARGET := ~/target/aarch64-unknown-none/release/${KERN}
OBJCPY := cargo objcopy -- --strip-all -O binary
TTY_PATH := /dev/ttyUSB0
BUILD := ~/build

.PHONY: all build qemu transmit objdump nm check clean install test

all: build

build:
	@echo "+ Building build/$(KERN).elf [xbuild/$@]"
	@cargo xbuild --release --target-dir ~/target
	@mkdir -p build
	@cp -f $(TARGET) $(BUILD)/$(KERN).elf

	@echo "+ Building build/$(KERN).bin [objcopy]"
	@$(OBJCPY) $(TARGET) $(BUILD)/$(KERN).bin

check:
	@cargo xcheck

qemu: build
	./qemu.sh $(BUILD)/$(KERN).bin

transmit: build
	@echo "+ Transmitting build/$(KERN).bin to $(TTY_PATH)"
	ttywrite -i $(BUILD)/$(KERN).bin $(TTY_PATH)

objdump: build
	cargo objdump -- -disassemble -no-show-raw-insn -print-imm-hex $(BUILD)/$(KERN).elf

nm: build
	cargo nm $(BUILD)/$(KERN).elf

clean:
	cargo clean
	rm -rf $(BUILD)

install: build
	@echo "+ Installing build/$(KERN).elf [install-kernel.py]"
	@$(ROOT)/bin/install-kernel.py $(BUILD)/$(KERN).elf

test:
	cargo test --target=$(shell $(ROOT)/bin/get-host-target.sh)