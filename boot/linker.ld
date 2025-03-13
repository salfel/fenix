MEMORY {
    sram (rwx) : ORIGIN = 0x402F0400, LENGTH = 0xFFFF
}

/* Section variables */
ENTRY(start)
SECTIONS {
  .text : { *(.text*) } > sram
  .bss : { *(.bss*) } > sram
  .data : { *(.data*) } > sram

  . = ALIGN(8);
  stack_start = .;
  . += 16K;
  stack_end = .;

  . = ALIGN(8);
  user_stack_start = .;
  . += 16K;
  user_stack_end = .;

  . = ALIGN(8);
  irq_stack_start = .;
  . += 4K;
  irq_stack_end = .;

  _end = . ;
}
