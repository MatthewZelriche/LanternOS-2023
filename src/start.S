.section ".text.boot"
.global _start

// For now, just sets a register for debugging purposes & loops
_start:
   movz x0, 0xbeef
   movk x0, 0xdead, lsl 16

   loop:
   bl loop
