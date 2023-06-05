BOOTLOADER := bootloader/opensbi-v1.2-2m.bin


KERNEL_ELF := kernel/target/riscv64imac-unknown-none-elf/release/kernel
KERNEL_DEBUG_ELF :=kernel/target/riscv64imac-unknown-none-elf/debug/kernel

KERNEL_BIN := kernel/target/riscv64imac-unknown-none-elf/release/kernel.bin
KERNEL_DEBUG_BIN := kernel/target/riscv64imac-unknown-none-elf/debug/kernel.bin

# KERNEL_ENTRY := 0x80200000

MEMORY_SIZE := 128M

CPU_NUM := 2

DRIVE_FILE := sdcard.img

clean:
	cd kernel && cargo clean

build:
	cd kernel && cargo build --offline
	# rust-objcopy --strip-all $(KERNEL_DEBUG_ELF) -O binary $(KERNEL_DEBUG_BIN)

copy_cargo:
	cp -r kernel/cargo kernel/.cargo

offline:copy_cargo
	cd kernel && cargo build --release --offline

release:
	cd kernel && cargo build --release
	# rust-objcopy --strip-all $(KERNEL_ELF) -O binary $(KERNEL_BIN)

qemu:release
	@cp ~/Music/sdcard.img ./sdcard.img
	@qemu-system-riscv64 \
		-machine virt \
    	-nographic \
		-kernel $(KERNEL_ELF)\
    	-bios $(BOOTLOADER) \
		-m $(MEMORY_SIZE) \
		-smp $(CPU_NUM)\
    	-drive file=$(DRIVE_FILE),if=none,format=raw,id=x0\
		-device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0
gdb:build
	@cp ~/Music/sdcard.img ./sdcard.img
	@tmux new-session -d \
		"qemu-system-riscv64 \
			-machine virt \
    		-nographic \
			-kernel $(KERNEL_DEBUG_ELF)\
    		-bios $(BOOTLOADER) \
			-m $(MEMORY_SIZE) \
			-smp $(CPU_NUM) \
			-drive file=$(DRIVE_FILE),if=none,format=raw,id=x0\
			-device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0\
    		-s -S" && \
    tmux split-window -h \
		"riscv64-unknown-elf-gdb \
			-ex 'file $(KERNEL_DEBUG_ELF)' \
    		-ex 'set arch riscv:rv64' \
    		-ex 'target remote localhost:1234'" &&\
	tmux -2 attach-session -d

gdb2:release
	@tmux new-session -d \
		"qemu-system-riscv64 \
			-machine virt \
    		-nographic \
			-kernel $(KERNEL_ELF)\
    		-bios $(BOOTLOADER) \
			-m $(MEMORY_SIZE) \
			-smp $(CPU_NUM) \
			-drive file=$(DRIVE_FILE),if=none,format=raw,id=x0\
			-device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0\
    		-s -S" && \
    tmux split-window -h \
		"riscv64-unknown-elf-gdb \
			-ex 'file $(KERNEL_DEBUG_ELF)' \
    		-ex 'set arch riscv:rv64' \
    		-ex 'target remote localhost:1234'" &&\
	tmux -2 attach-session -d

all:offline
	cp $(KERNEL_ELF) kernel-qemu
	cp $(BOOTLOADER) sbi-qemu



