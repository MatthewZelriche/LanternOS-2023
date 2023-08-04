INCLUDE common.ld

ENTRY (main)

SECTIONS
{
    . = 0x1000000000;
    __kernel_start = .;
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