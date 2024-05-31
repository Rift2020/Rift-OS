BOOTLOADER := bootloader/opensbi-v1.2-2m.bin


KERNEL_ELF := kernel/target/riscv64imac-unknown-none-elf/release/kernel
KERNEL_DEBUG_ELF :=kernel/target/riscv64imac-unknown-none-elf/debug/kernel

KERNEL_BIN := kernel/target/riscv64imac-unknown-none-elf/release/kernel.bin
KERNEL_DEBUG_BIN := kernel/target/riscv64imac-unknown-none-elf/debug/kernel.bin

# KERNEL_ENTRY := 0x80200000

MEMORY_SIZE := 128M

CPU_NUM := 2

OSCOMP2023_PRE_DRIVE:= sdcard-pre.img
OSCOMP2023_FINAL_DRIVE := sdcard-final.img
EXAMPLE_DRIVE := sdcard-example.img
DRIVE_FILE := $(EXAMPLE_DRIVE)
DRIVE_FILE_COPY := sdcard_copy.img

TFTP_SHARE := /tftp_share

VF_COM := /dev/ttyUSB0

dl_img:
	wget -qO-  https://github.com/oscomp/testsuits-for-oskernel/raw/6efdff31d9eb882ff73352e7010019fc56ead4de/sdcard.img.gz | gunzip - > $(OSCOMP2023_PRE_DRIVE)
	wget -qO-  https://github.com/oscomp/testsuits-for-oskernel/releases/download/2023-final/sdcard.img.gz | gunzip - > $(OSCOMP2023_FINAL_DRIVE)
	
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
	@cp $(DRIVE_FILE) $(DRIVE_FILE_COPY)
	@qemu-system-riscv64 \
		-machine virt \
    	-nographic \
		-kernel $(KERNEL_ELF)\
    	-bios $(BOOTLOADER) \
		-m $(MEMORY_SIZE) \
		-smp $(CPU_NUM)\
    	-drive file=$(DRIVE_FILE_COPY),if=none,format=raw,id=x0\
		-device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0
gdb:build
	@cp $(DRIVE_FILE) $(DRIVE_FILE_COPY)
	@tmux new-session -d \
		"qemu-system-riscv64 \
			-machine virt \
    		-nographic \
			-kernel $(KERNEL_DEBUG_ELF)\
    		-bios $(BOOTLOADER) \
			-m $(MEMORY_SIZE) \
			-smp $(CPU_NUM) \
			-drive file=$(DRIVE_FILE_COPY),if=none,format=raw,id=x0\
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

tftp:release
	cp $(KERNEL_BIN) $(TFTP_SHARE)/os.bin

vf2:tftp
	sudo minicom -D $(VF_COM) -b 115200

