arch ?= x86_64
kernel := build/kernel-$(arch).bin
iso := build/hlq-os-$(arch).iso

# Path to your compiled Rust static library
# rust_os := target/x86_64-hlq_os/debug/libHLQ_OS.a
rust_os := target/x86_64-unknown-none/debug/libHLQ_OS.a

linker_script := src/arch/$(arch)/linker.ld
grub_cfg := src/arch/$(arch)/grub.cfg
assembly_source_files := $(wildcard src/arch/$(arch)/*.asm)
assembly_object_files := $(patsubst src/arch/$(arch)/%.asm, \
    build/arch/$(arch)/%.o, $(assembly_source_files))

.PHONY: all clean run iso kernel cargo

all: $(kernel)

clean:
	@rm -r build
	@cargo clean

run: $(iso)
	@qemu-system-x86_64 -cdrom $(iso)

iso: $(iso)

$(iso): $(kernel) $(grub_cfg)
	@mkdir -p build/isofiles/boot/grub
	@cp $(kernel) build/isofiles/boot/kernel.bin
	@cp $(grub_cfg) build/isofiles/boot/grub
	@grub-mkrescue -o $(iso) build/isofiles 2> /dev/null
	@rm -r build/isofiles

# 1. Force cargo to check for changes every build
cargo:
	@cargo build --target x86_64-unknown-none
# 	@cargo build --target x86_64-hlq_os.json -Zjson-target-spec -Zbuild-std=core,compiler_builtins -Zbuild-std-features=compiler-builtins-mem

# 2. Add $(rust_os) as a dependency and link it
$(kernel): cargo $(assembly_object_files) $(linker_script)
	@ld -n -T $(linker_script) -o $(kernel) $(assembly_object_files) $(rust_os)

# compile assembly files
build/arch/$(arch)/%.o: src/arch/$(arch)/%.asm
	@mkdir -p $(shell dirname $@)
	@nasm -felf64 $< -o $@