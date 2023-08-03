.section ".text.boot"
.global _start

// Taken from rpi4os.com
// Will study switching exception levels more 
// closely in the future
.equ CR_EL3, 818944004
.equ CPTR, 13311
.equ CPACR_EL1, 3145728
.equ HCR, 2147483648
.equ SCR, 1073
.equ SPSR, 453

// For now, just sets a register for debugging purposes & loops
_start:
   // Get core number. We support up to 4 cores
   // Stall all cores except core 0
   mrs     x1, mpidr_el1
   and     x1, x1, #3
   cbz     x1, to_el1
   stall_core:
   bl stall_core     

   // Transition from EL3 -> EL1
   to_el1:
   // Don't trap to EL3 for fp access
   // don't trap on fp instruction access for EL1 and EL0
   ldr x1, =CPTR
   msr cptr_el3, x1
   ldr x1, =CPACR_EL1
   msr cpacr_el1, x1	

   // eret to EL1
   ldr x1, =CR_EL3
   msr sctlr_el1, x1
   ldr x1, =HCR
   msr hcr_el2, x1
   ldr x1, =SCR
   msr scr_el3, x1
   ldr x1, =SPSR
   msr spsr_el3, x1
   adr x1, setup_stack	
   msr elr_el3, x1
   mrs x6, elr_el3
   eret

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
