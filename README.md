<h1 align="center">LanternOS</h1>

## About

LanternOS is a hobbyist 64-bit multitasking kernel that currently supports the Raspberry Pi 3 and Raspberry Pi 4
devices.

## Building

### Disclaimer: This has never been tested on real hardware. Run this on real hardware at your own risk. It is highly recommended to use an emulator such as QEMU.

Build requirements:

- Latest Rust Nightly
- Make
- Cargo Binutils (`cargo install cargo-binutils`)
- `qemu-system-aarch64`, if you do not have access to real hardware

Note that you will also need a file named `card.img` in the `out/` directory that contains a FAT filesystem. This can be created for example with `dd` and `mformat`.

Once the build requirements are met, building the project should be as simple as running `make`. This
will produce an `out` directory, containing the resulting binaries. These can be installed onto an SD Card
and used in a real Raspberry Pi, or you can use qemu. To use qemu, run:

`make qemu-raspi3`

to run on a RPI3 VM. Qemu does not officially support the RPI4, but [patches to provide support exist](https://gitlab.com/qemu-project/qemu/-/issues/1208#note_1435620155). If you build qemu yourself with these patches, you can run this project in a RPI3 VM via:

`make qemu QEMU_PATH=<path-to-qemu>`.

## Roadmap

- [x] Multi-stage bootloader to load kernel ELF into memory
- [x] Initialize MMU and load kernel into higher half address space
- [x] Implement synchronization primitives (`Mutex`, `Barrier`)
- [x] Initialize secondary cores
- [x] Implement statically-sized kernel heap and enable `alloc` crate
- [x] Simple Read and write to FAT filesystem on SDCard
- [ ] Implement HAL to allow for easier porting across architectures other than ARM
- [ ] Automate building of `card.img` for use in qemu
- [ ] Define Syscall infrastructure
- [ ] Load userspace applications into lower half
- [ ] Create `init` userspace process (probably a shell)
- [ ] Basic Process scheduling
- [ ] Framebuffer driver
- [ ] Font rendering
- [ ] Graphical TTY to replace UART as primary output
- [ ] Dynamic linking support
- [ ] Research a bare minimum port of libc and possibly rust's std
- [ ] [But can it run Doom?](https://github.com/ozkl/doomgeneric)

## Licensing Information

The kernel and bootloader are licensed under the MIT License.

Please see `license.html` for a full list of licensing information, including third-party licenses.

## Sample Output

What follows is an example output of the kernel log via UART, which is currently the primary method
of interfacing with the OS at this time:

```
[0.09980] Raspi bootloader is preparing environment for kernel...
[0.10450] Successfully parsed kernel ELF
[0.10524] Loaded Kernel ELF into memory
[0.10543] Initializing page frame allocator...
[3.46555] Successfully initialized page frame allocator with 1015650 free frames.
[3.46689] Mapped kernel to higher half range 0xffff000000000000 - 0xffff000000026000
[3.46743] Mapped four kernel stacks of size 0x2000 bytes
[3.46767] Mapped physical memory into higher half starting at address: 0xffff000040000000
[3.46790] Printing memory map:

Page size:       4.000 KiB
Reserved Pages:  32932
Available Pages: 1015644
Total Memory:    4.000 GiB
Avail Memory:    3.874 GiB

Type: Firmware   | 0x0000000000000000 - 0x0000000000001000 | 4.000 KiB
Type: Stack      | 0x0000000000001000 - 0x0000000000009000 | 32.000 KiB
Type: Kernel     | 0x0000000000009000 - 0x000000000002f000 | 152.000 KiB
Type: Free       | 0x000000000002f000 - 0x000000000007f000 | 320.000 KiB
Type: Bootloader | 0x000000000007f000 - 0x00000000000ce000 | 316.000 KiB
Type: Free       | 0x00000000000ce000 - 0x0000000008000000 | 127.195 MiB
Type: DeviceTree | 0x0000000008000000 - 0x0000000008020000 | 128.000 KiB
Type: Free       | 0x0000000008020000 - 0x000000003c000000 | 831.875 MiB
Type: Firmware   | 0x000000003c000000 - 0x0000000040000000 | 64.000 MiB
Type: Free       | 0x0000000040000000 - 0x00000000fbffa000 | 2.937 GiB
Type: BLReserved | 0x00000000fbffa000 - 0x00000000fc000000 | 24.000 KiB
Type: MMIO       | 0x00000000fc000000 - 0x0000000100000000 | 64.000 MiB

[3.47222] Bootloader allocated 6 pages of memory in total
[3.47264] Successfully enabled the MMU
[3.47304] Initializng secondary cores and transferring control to kernel entry point...

[3.47420] Performing kernel early init...
[3.47533] Registered exception handlers at 0xffff00000000b800
[4.07924] Initialized page frame allocator with 1015723 free frames
[4.08208] Initialized kernel heap at address range 0xffff000000032000 - 0xffff000000232000
[4.08587] Initialized EMMC2 driver. Storage medium ready to receive block requests.
[4.09861] Successfully read FAT filesystem from SDCard
[4.09895] Kernel initialization complete
[4.09916] Writing 'Hello, world!' to file 'hello.txt' at root dir
[4.10560] Waiting 5 seconds...
[9.10592] Opening file 'hello.txt' and reading file contents
[9.10836] Read 13 bytes from file. File contents: Hello, world!
[9.10908 | Core 3] Hello from secondary core!
[9.10941 | Core 2] Hello from secondary core!
[9.10950 | Core 1] Hello from secondary core!

KERNEL PANIC!
Location: libs/arch/raspi/exception/src/lib.rs:73:5
Reason:
Uncaught exception! Dumping CPU State:

Exception Syndrome: 0x96000004
Faulting Address: 0xdeadbeef
Saved Program Status: 0x200001c5
Exception Link Register: 0xffff0000000048a8
Link Register: 0xffff000000004890
Stack Pointer: 0xffff000000028430

General Purpose Registers:
X00: 0x0000000000000000  X01: 0x00000000000f4240
X02: 0x0000000000000001  X03: 0xffff00000000af58
X04: 0x0000000000000000  X05: 0x0000000000000002
X06: 0x000009eecbfb11e4  X07: 0x0000400000000000
X08: 0x00000000deadbeef  X09: 0xffff000000025af8
X10: 0x0000000000000000  X11: 0x0000000000000000
X12: 0x000000000000000a  X13: 0x0000000000000090
X14: 0xffff00000001f77a  X15: 0xffff000000024cf8
X16: 0xffff000000024df8  X17: 0x0000000000000005
X18: 0x0000000000000021  X19: 0xffff000000028b68
X20: 0xffff000000028af0  X21: 0xffff000000028b80
X22: 0xffff00000001f9a8  X23: 0xffff000000025a68
X24: 0xffff000000025a58  X25: 0x0000000000000000
X26: 0xffff000000231000  X27: 0x00000000fbdf9000
X28: 0xffff000000232000  X29: 0xffff000000025a88
X30: 0xffff000000004890

Q00: 0x000000000000f424  Q01: 0x0000000000000000
Q02: 0x0000000000000000  Q03: 0x0000000000000000
Q04: 0x0000000000000000  Q05: 0x0000000000000000
Q06: 0x0000000000000000  Q07: 0x0000000000000000
Q08: 0x0000000000000000  Q09: 0x0000000000000000
Q10: 0x0000000000000000  Q11: 0x0000000000000000
Q12: 0x0000000000000000  Q13: 0x0000000000000000
Q14: 0x0000000000000000  Q15: 0x0000000000000000
Q16: 0x0000000000000000  Q17: 0x0000000000000000
Q18: 0x0000000000000000  Q19: 0x0000000000000000
Q20: 0x0000000000000000  Q21: 0x0000000000000000
Q22: 0x0000000000000000  Q23: 0x0000000000000000
Q24: 0x0000000000000000  Q25: 0x0000000000000000
Q26: 0x0000000000000000  Q27: 0x0000000000000000
Q28: 0x0000000000000000  Q29: 0x0000000000000000
Q30: 0x0000000000000000  Q31: 0x0000000000000000
```
