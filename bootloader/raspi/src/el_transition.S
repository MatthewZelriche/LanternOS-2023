// Taken from rpi4os.com
// Will study switching exception levels more 
// closely in the future
.equ CR_EL3, 818944004
.equ CPTR, 13311
.equ CPACR_EL1, 3145728
.equ HCR, 2147483648
.equ SPSR, 453

.macro el_transition label
   // Don't trap to EL3 for fp access
   // don't trap on fp instruction access for EL1 and EL0
   ldr x1, =CPTR
   msr cptr_el2, x1
   ldr x1, =CPACR_EL1
   msr cpacr_el1, x1	

   // eret to EL1
   ldr x1, =CR_EL3
   msr sctlr_el1, x1
   ldr x1, =HCR
   msr hcr_el2, x1
   ldr x1, =SPSR
   msr spsr_el2, x1
   adr x1, \label\()
   msr elr_el2, x1
   mrs x6, elr_el2
   eret
.endmacro