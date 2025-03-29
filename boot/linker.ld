MEMORY {
    sram (rwx) : ORIGIN = 0x402F0400, LENGTH = 0xFFFF
}

ENTRY(_start)
SECTIONS {
    .text : { 
        *(.text._start)
        *(.text*)
    } > sram

    .bss : { 
        *(.bss*)
    } > sram

    .data : { 
        *(.data*)
    } > sram

    . = ALIGN(4);
    heap_start = .;
    . += 4K;
    heap_end = .;

    . = ALIGN(8);
    stack_start = .;
    . += 4K;
    stack_end = .;

    . = ALIGN(8);
    irq_stack_start = .;
    . += 4K;
    irq_stack_end = .;

    _end = .;
}
