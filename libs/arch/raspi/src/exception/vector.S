.align 11

.globl exception_vectors
exception_vectors:
.org 0x200
   b current_elx_synchronous_stub
.org 0x280
   b current_elx_irq_stub
.org 0x300
   b current_elx_fiq_stub
.org 0x380
   b current_elx_serror_stub


