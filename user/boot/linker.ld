MEMORY {
    ram (rwx) : ORIGIN = 0x0, LENGTH = 0x1000
}

ENTRY(_start)
SECTIONS {
    .text : { 
        *(.text._start)
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
