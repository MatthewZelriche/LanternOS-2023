

.text
.type init_secondary_core, @function
.globl init_secondary_core
init_secondary_core: 
   ldr x5, =core_start
   str x5, [x0]   // First arg contains mailbox addr of core
   sev            // MMU must already be enabled by the time this fn is called
   ret



.text
.type core_start, @function
.globl core_start
core_start:
   el_transition setup_secondary_stack

   setup_secondary_stack:
   //mov sp, x1 // second arg is address of top of stack
   bl setup_secondary_stack