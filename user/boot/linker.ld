MEMORY {
    ram (rwx) : ORIGIN = 0x0, LENGTH = 0x1000
}

ENTRY(__wrap__start)
SECTIONS {
    .text : { 
        *(.text*)
    } > ram

    .bss : { 
        *(.bss*)
    } > ram

    .data : { 
        *(.data*)
    } > ram

    _end = .;
}
