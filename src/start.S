.section ".text.boot"
.global _start

// For now, just sets a register for debugging purposes & loops
_start:
   // Set the start of the stack
   ldr x0, =__stack
   mov sp, x0

   ldr x0, =0xdeadbeef

   // Jump to kernel main
   bl main

   // If for some reason we ever accidentally return from kernel main, spin...
   loop:
   bl loop
