BOOTLOADER := bootloader/opensbi-v1.2.bin


KERNEL_ELF := kernel/target/riscv64imac-unknown-none-elf/release/kernel
KERNEL_DEBUG_ELF :=kernel/target/riscv64imac-unknown-none-elf/debug/kernel

KERNEL_BIN := kernel/target/riscv64imac-unknown-none-elf/release/kernel.bin
KERNEL_DEBUG_BIN := kernel/target/riscv64imac-unknown-none-elf/debug/kernel.bin

KERNEL_ENTRY := 0x80200000

MEMORY_SIZE := 128M

CPU_NUM := 2

DRIVE_FILE := sdcard.img

build:
	cd kernel && cargo build
	rust-objcopy --strip-all $(KERNEL_DEBUG_ELF) -O binary $(KERNEL_DEBUG_BIN)

release:
	cd kernel && cargo build --release
	rust-objcopy --strip-all $(KERNEL_ELF) -O binary $(KERNEL_BIN)

qemu:release
	@qemu-system-riscv64 \
		-machine virt \
    	-nographic \
    	-bios $(BOOTLOADER) \
		-m $(MEMORY_SIZE) \
		-smp $(CPU_NUM)\
    	-device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY)\
		-drive file=$(DRIVE_FILE),if=none,format=raw,id=x0\
		-device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0
gdb:build
	@tmux new-session -d \
		"qemu-system-riscv64 \
			-machine virt \
    		-nographic \
    		-bios $(BOOTLOADER) \
			-m $(MEMORY_SIZE) \
			-smp $(CPU_NUM) \
    		-device loader,file=$(KERNEL_DEBUG_BIN),addr=$(KERNEL_ENTRY)\
			-drive file=$(DRIVE_FILE),if=none,format=raw,id=x0\
			-device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0\
    		-s -S" && \
    tmux split-window -h \
		"riscv64-unknown-elf-gdb \
			-ex 'file $(KERNEL_DEBUG_ELF)' \
    		-ex 'set arch riscv:rv64' \
    		-ex 'target remote localhost:1234'" &&\
	tmux -2 attach-session -d




