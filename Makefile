
QEMU_PATH=

.PHONY: clean kernel kernel-dbg qemu

kernel:
	cargo build --release --target aarch64-unknown-none
	mkdir -p out/
	cargo objcopy --release --target aarch64-unknown-none -- -O binary out/kernel8.img


kernel-dbg:
	cargo build --target aarch64-unknown-none
	mkdir -p out/
	cargo objcopy --target aarch64-unknown-none -- out/debug.img

qemu: kernel-dbg
	$(QEMU_PATH)qemu-system-aarch64 -M raspi4b8g -kernel out/debug.img -monitor stdio

clean:
	cargo clean
	rm -rf out/