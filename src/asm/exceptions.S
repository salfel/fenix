.global setup_exceptions

setup_exceptions:
    ldr r0, irq_handler_addr
    ldr r1, _irq_handler
    str r1, [r0]

    ldr r0, swi_handler_addr
    ldr r1, _swi_handler
    str r1, [r0]

    ldr r0, data_abort_addr
    ldr r1, _data_abort
    str r1, [r0]

    ldr r0, fetch_abort_addr
    ldr r1, _fetch_abort
    str r1, [r0]

    ldr r0, undefined_addr
    ldr r1, _undefined
    str r1, [r0]

    bx lr


irq_handler_addr: .word 0x4030CE38
swi_handler_addr: .word 0x4030CE28
data_abort_addr: .word 0x4030CE30
fetch_abort_addr: .word 0x4030CE2C
undefined_addr: .word 0x4030CE24

_irq_handler: .word irq_handler
_swi_handler: .word handle_swi
_data_abort: .word data_abort_handler
_fetch_abort: .word fetch_abort_handler
_undefined: .word undefined_handler
