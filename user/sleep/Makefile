ROOT := $(shell git rev-parse --show-toplevel)

BIN := $(shell basename $(shell realpath .))
TARGET := ~/target/$(BIN)/aarch64-unknown-none/release/$(BIN)
BUILD := ~/build/$(BIN)
OBJCPY := cargo objcopy -- --strip-all -O binary

.PHONY: all build qemu objdump nm clean

all: build

build:
	@echo "+ Building build/$(BIN).elf [xbuild/$@]"
	@cargo xbuild --release --target-dir ~/target/$(BIN)
	@mkdir -p $(BUILD)
	@cp -f $(TARGET) $(BUILD)/$(BIN).elf

	@echo "+ Building $(BUILD)/$(BIN).bin [objcopy]"
	@$(OBJCPY) $(TARGET) $(BUILD)/$(BIN).bin

check:
	@cargo xcheck

objdump: build
	cargo objdump -- -disassemble -no-show-raw-insn -print-imm-hex $(BUILD)/$(BIN).elf

nm: build
	cargo nm $(BUILD)/$(BIN).elf

clean:
	cargo clean
	rm -rf $(BUILD)
