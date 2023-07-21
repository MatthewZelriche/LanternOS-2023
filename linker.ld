SECTIONS
{
    . = 0x7FFF0;
    __stack = .;
    /* RPI4 expects boot code at this addr */
    . = 0x80000;
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