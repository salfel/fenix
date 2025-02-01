MEMORY {
    sram (rwx) : ORIGIN = 0x402F0400, LENGTH = 0xFFFF
}

/* Section variables */
ENTRY(start)
SECTIONS {
  .text : { *(.text*) } > sram
  .bss : { *(.bss*) } > sram
  .data : { *(.data*) } > sram
  _end = . ;
}
