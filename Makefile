BOOTLOADER := bootloader/opensbi-v1.2.bin


KERNEL_ELF := kernel/target/riscv64gc-unknown-none-elf/release/kernel
KERNEL_BIN := kernel/target/riscv64gc-unknown-none-elf/release/kernel.bin
KERNEL_ENTRY := 0x80200000


build:
	cd kernel && cargo build --release
	rust-objcopy --strip-all $(KERNEL_ELF) -O binary $(KERNEL_BIN)

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




