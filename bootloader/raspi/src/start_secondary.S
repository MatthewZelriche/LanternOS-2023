.text
.type init_secondary_core, @function
.globl init_secondary_core
init_secondary_core: 
   str x1, [x0]   // First arg contains mailbox addr of core
                  // Second arg contains address of entry point

   sev            // MMU must already be enabled by the time this fn is called
   ret


.macro to_rust_main core
   ldr x1, [x0]
   mov sp, x1

   add x10, x0, #8      // TTBR0 ptr
   add x11, x0, #16     // TTBR1 ptr
   // Store ttbr0 and ttbr1 ptr into arguments for rust fn
   ldr x0, =\core\()
   ldr x1, [x10]
   ldr x2, [x11]
   bl secondary_core_main
.endmacro


.text
.type core_1_start, @function
.globl core_1_start
core_1_start:
   // We can't pass arguments from init_secondary_core to a new core, so we hardcode in memory regions
   // in our first page for each core. These will contain the arguments.
   // These regions are defined in main.rs in the raspi bootloader
   ldr x0, =0xFA0

   el_transition core_1_el1

   core_1_el1:
   to_rust_main 1


.text
.type core_2_start, @function
.globl core_2_start
core_2_start:
   // We can't pass arguments from init_secondary_core to a new core, so we hardcode in memory regions
   // in our first page for each core. These will contain the arguments.
   // These regions are defined in main.rs in the raspi bootloader
   ldr x0, =0xFC0

   el_transition core_2_el1

   core_2_el1:
   to_rust_main 2


.text
.type core_3_start, @function
.globl core_3_start
core_3_start:
   // We can't pass arguments from init_secondary_core to a new core, so we hardcode in memory regions
   // in our first page for each core. These will contain the arguments.
   // These regions are defined in main.rs in the raspi bootloader
   ldr x0, =0xFE0

   el_transition core_3_el1

   core_3_el1:
   to_rust_main 3