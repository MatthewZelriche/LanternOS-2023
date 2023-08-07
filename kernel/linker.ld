INCLUDE common.ld

ENTRY (main)

SECTIONS
{
    . = __KERNEL_VIRT_START;
    .text :
    {
      *(.text .text.*)
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
    .bss :
    {
        *(.bss)
    }
}