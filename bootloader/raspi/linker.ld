INCLUDE common.ld

SECTIONS
{
    . = 0x6E000; /* 12kib kernel stack */
    __stack_end = .;
    /* RPI4 expects boot code at this addr */
    /* We also start our stack here */
    . = 0x80000;
    __stack = .;
    __bootloader_start = .;
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
    __bootloader_end = .;
    __bss_size = __bss_end - __bss_start;
}