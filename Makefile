BOOTBIN=./target/x86_64-rust_os/debug/bootimage-rust_os.bin

bootimage:
	bootimage build

boot:
	bootimage run

bootbr:
	bootimage build && bootimage run -- -serial mon:stdio -device isa-debug-exit,iobase=0xf4,iosize=0x04 -display none

bootv2: $(BOOTBIN)
	qemu-system-x86_64 -drive format=raw,file=$(BOOTBIN) -serial mon:stdio
