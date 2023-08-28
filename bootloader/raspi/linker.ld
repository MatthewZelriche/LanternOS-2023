INCLUDE common.ld

SECTIONS
{
    . = 0x7FF00; /* 4kib kernel stack */
    __BL_STACK_END = .;
    /* RPI4 expects boot code at this addr */
    /* We also start our stack here */
    . = 0x80000;
    __BL_STACK = .;
    __BL_START = .;
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
    __BL_END = .;
    __bss_size = __bss_end - __bss_start;
}