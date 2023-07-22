.section ".text.boot"
.global _start

// For now, just sets a register for debugging purposes & loops
_start:
   // Set the start of the stack
   ldr x0, =__stack
   mov sp, x0

   // zero out the BSS
   ldr x0, =__bss_start
   ldr x1, =__bss_size
   loop_bss:
   cbz x1, jump_kernel
   str xzr, [x0]
   add x0, x0, #8
   sub x1, x1, #8
   bl loop_bss


   // Jump to kernel main
   jump_kernel:
   bl main

   // If for some reason we ever accidentally return from kernel main, spin...
   loop:
   bl loop
