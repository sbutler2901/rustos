BOOTBIN=./target/x86_64-rust_os/debug/bootimage-rust_os.bin

bootimage:
	bootimage build

boot:
	bootimage run

bootv2: $(BOOTBIN)
	qemu-system-x86_64 -drive format=raw,file=$(BOOTBIN)