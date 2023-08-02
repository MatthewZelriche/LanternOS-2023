SECTIONS
{
    . = 0x7D120; /* 12kib kernel stack */
    __stack_end = .;
    /* RPI4 expects boot code at this addr */
    /* We also start our stack here - eventually we will need a stack guard */
    . = 0x80000;
    __stack = .;
    .text :
    {
      KEEP(*(.text.boot)) *(.text .text.*)
    }
    . = ALIGN(16);
    .rodata :
    {
     	*(.rodata .rodata.*)
    }
    . = ALIGN(16);
    .data :
    {
     	*(.data)
    }
    . = ALIGN(16);
    /* BSS info needed to zero out the BSS section, as expected */
    .bss :
    {
      __bss_start = .;
        *(.bss)
      __bss_end = .;
    }
    __kernel_end = .;
    __bss_size = __bss_end - __bss_start;
}