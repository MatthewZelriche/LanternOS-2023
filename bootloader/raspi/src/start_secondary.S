

.text
.type init_secondary_core, @function
.globl init_secondary_core
init_secondary_core: 
   str x1, [x0]   // First arg contains mailbox addr of core
                  // Second arg contains address of entry point

   sev            // MMU must already be enabled by the time this fn is called
   ret


.text
.type core_1_start, @function
.globl core_1_start
core_1_start:
   // We can't pass arguments from init_secondary_core to a new core, so we hardcode in memory regions
   // in our first page for each core. These will contain the arguments.
   // These regions are defined in main.rs in the raspi bootloader
   ldr x0, =0xFA0

   el_transition setup_core_1_stack

   setup_core_1_stack:
   ldr x1, [x0]
   mov sp, x1
   core_1_loop:
   bl core_1_loop


.text
.type core_2_start, @function
.globl core_2_start
core_2_start:
   // We can't pass arguments from init_secondary_core to a new core, so we hardcode in memory regions
   // in our first page for each core. These will contain the arguments.
   // These regions are defined in main.rs in the raspi bootloader
   ldr x0, =0xFC0

   el_transition setup_core_2_stack

   setup_core_2_stack:
   ldr x1, [x0]
   mov sp, x1
   core_2_loop:
   bl core_2_loop


.text
.type core_3_start, @function
.globl core_3_start
core_3_start:
   // We can't pass arguments from init_secondary_core to a new core, so we hardcode in memory regions
   // in our first page for each core. These will contain the arguments.
   // These regions are defined in main.rs in the raspi bootloader
   ldr x0, =0xFE0

   el_transition setup_core_3_stack

   setup_core_3_stack:
   ldr x1, [x0]
   mov sp, x1
   core_3_loop:
   bl core_3_loop