BOOTLOADER := bootloader/opensbi-v1.2.bin

KERNEL_ELF := target/riscv64gc-unknown-none-elf/release/os
KERNEL_BIN := kernel/target/riscv64gc-unknown-none-elf/release/os.bin
KERNEL_ENTRY := 0x80200000


build:
	cd kernel && cargo build --release

qemu:build
	@qemu-system-riscv64 \
		-machine virt \
    	-nographic \
    	-bios $(BOOTLOADER) \
    	-device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY)\

gdb:build
	@tmux new-session -d \
		"qemu-system-riscv64 \
			-machine virt \
    		-nographic \
    		-bios $(BOOTLOADER) \
    		-device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY)\
    		-s -S" && \
    tmux split-window -h \
		"riscv64-unknown-elf-gdb \
			-ex 'file $(KERNEL_ELF)' \
    		-ex 'set arch riscv:rv64' \
    		-ex 'target remote localhost:1234'" &&\
	tmux -2 attach-session -d




