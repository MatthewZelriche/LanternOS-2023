
QEMU_PATH=
DTB_RASPI4=vendor/bcm2711-rpi-4-b.dtb
DTB_RASPI3=vendor/bcm2710-rpi-3-b.dtb

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
	cargo objcopy --bin bootloader-raspi --target aarch64-unknown-none -- -O binary out/kernel8.img

qemu: kernel
	$(QEMU_PATH)qemu-system-aarch64 -M raspi4b4g -kernel out/kernel8.img -serial stdio -dtb $(DTB_RASPI4)

qemu-raspi3: kernel
	$(QEMU_PATH)qemu-system-aarch64 -M raspi3b -kernel out/kernel8.img -serial stdio -dtb $(DTB_RASPI3)

clean:
	cargo clean
	rm -rf out/