
QEMU_PATH=

.PHONY: clean kernel kernel-dbg qemu

kernel:
	cargo build --release --bin kernel --target aarch64-unknown-none
	mkdir -p out/
	cargo objcopy --release --bin kernel --target aarch64-unknown-none -- out/lantern-os.elf
	cargo build --release --bin bootloader-raspi --target aarch64-unknown-none
	cargo objcopy --release --bin bootloader-raspi --target aarch64-unknown-none -- -O binary out/kernel8.img

kernel-dbg:
	cargo build --bin kernel --target aarch64-unknown-none
	mkdir -p out/
	cargo objcopy --bin kernel --target aarch64-unknown-none -- out/lantern-os.elf
	cargo build --bin bootloader-raspi --target aarch64-unknown-none
	mkdir -p out/
	cargo objcopy --bin bootloader-raspi --target aarch64-unknown-none -- out/kernel8.img

qemu: kernel-dbg
	$(QEMU_PATH)qemu-system-aarch64 -M raspi4b4g -kernel out/kernel8.img -serial stdio

clean:
	cargo clean
	rm -rf out/