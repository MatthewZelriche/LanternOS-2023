.section ".text.boot"
.global _start

// For now, just sets a register for debugging purposes & loops
_start:
   // No need to park other cores, because if we run in QEMU with kernel8 as a flat
   // binary, the "hardware" parks the cores automatically until we wake them up with mailbox    

   // Transition from EL2 -> EL1
   // On real hardware (as well as QEMU when booting a raw binary),
   // execution starts in EL2
   el_transition setup_stack

   // Set the start of the stack
   setup_stack:
   ldr x1, =__stack
   mov sp, x1

   // zero out the BSS
   ldr x1, =__bss_start
   ldr x2, =__bss_size
   loop_bss:
   cbz x2, jump_bootloader
   str xzr, [x1]
   add x1, x1, #8
   sub x2, x2, #8
   bl loop_bss


   // Jump to bootloader main
   jump_bootloader:
   bl main

   // If for some reason we ever accidentally return from bootloader main, spin...
   loop:
   bl loop
