SECTIONS
{
    /* RPI4 expects boot code at this addr */
    . = 0x80000;
    .text :
    {
     	KEEP(*(.text.boot)) 
      *(.text)
    }
    .rodata :
    {
     	*(.rodata)
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