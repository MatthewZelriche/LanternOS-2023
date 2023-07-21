SECTIONS
{
    /* RPI4 expects boot code at this addr */
    /* We also start our stack here - eventually we will need a stack guard */
    . = 0x80000;
    __stack = .;
    .text :
    {
      KEEP(*(.text.boot)) *(.text .text.*)
    }
    .rodata :
    {
     	*(.rodata .rodata.*)
    }
    .data :
    {
     	*(.data)
    }
    .bss :
    {
      . = ALIGN(16);
        *(.bss)
    }
}