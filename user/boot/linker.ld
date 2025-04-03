MEMORY {
    sram (rwx) : ORIGIN = 0x0, LENGTH = 0x1000
}

ENTRY(_start)
SECTIONS {
    .text : { 
        *(.text*)
    } > sram

    .bss : { 
        *(.bss*)
    } > sram

    .data : { 
        *(.data*)
    } > sram

    _end = .;
}
