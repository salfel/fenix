.global setup_exceptions

setup_exceptions:
    ldr r0, data_abort_addr
    ldr r1, =data_abort_handler
    str r1, [r0]

    ldr r0, fetch_abort_addr
    ldr r1, =fetch_abort_handler
    str r1, [r0]

    ldr r0, undefined_addr
    ldr r1, =undefined_handler
    str r1, [r0]

    ldr r0, swi_addr
    ldr r1, =swi_handler
    str r1, [r0]

    ldr r0, irq_addr
    ldr r1, =irq_handler
    str r1, [r0]
    
    bx lr

data_abort_addr: .word 0x4030CE30
fetch_abort_addr: .word 0x4030CE2C
undefined_addr: .word 0x4030CE24
swi_addr: .word 0x4030CE28
irq_addr: .word 0x4030CE38
