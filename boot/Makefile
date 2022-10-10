ROOT := $(shell git rev-parse --show-toplevel)

KERN := boot
TARGET := ~/target/boot/aarch64-unknown-none/release/${KERN}
OBJCPY := cargo objcopy -- --strip-all -O binary
BUILD := ~/build/boot

.PHONY: all build qemu objdump nm check clean install test

all: build

build:
	@echo "+ Building build/$(KERN).elf [xbuild/$@]"
	@cargo xbuild --release --target-dir ~/target/boot/
	@mkdir -p $(BUILD)
	@cp -f $(TARGET) $(BUILD)/$(KERN).elf

	@echo "+ Building $(BUILD)/$(KERN).bin [objcopy]"
	@$(OBJCPY) $(TARGET) $(BUILD)/$(KERN).bin

check:
	@cargo xcheck

qemu: build
	./qemu.sh $(BUILD)/$(KERN).elf

objdump: build
	cargo objdump -- -disassemble -no-show-raw-insn -print-imm-hex $(BUILD)/$(KERN).elf

nm: build
	cargo nm $(BUILD)/$(KERN).elf

clean:
	cargo clean
	rm -rf $(BUILD)

install: build
	@echo "+ Installing $(BUILD)/$(KERN).elf [install-kernel.py]"
	@$(ROOT)/bin/install-kernel.py $(BUILD)/$(KERN).elf

test:
	cargo test --target=$(shell $(ROOT)/bin/get-host-target.sh)
