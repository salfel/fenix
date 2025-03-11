.global start

.extern setup
.extern setup_irq
.extern setup_swi

start:
    ldr r0, base_data_abort_addr
    ldr r1, data_abort
    str r1, [r0]

    ldr r0, base_fetch_abort_addr
    ldr r1, fetch_abort
    str r1, [r0]

    ldr r0, base_undefined_addr
    ldr r1, undefined
    str r1, [r0]

    bl setup
    bl setup_irq

    bl main
    b hang

    
handle_data_abort:
    bl data_abort_handler

    b hang

handle_fetch_abort:
    bl fetch_abort_handler

    b hang

handle_undefined:
    bl undefined_handler

    b hang

hang:
    b hang

stack_base: .word 0x4030BE30
base_data_abort_addr: .word 0x4030CE30
base_fetch_abort_addr: .word 0x4030CE2C
base_undefined_addr: .word 0x4030CE24
data_abort: .word handle_data_abort
fetch_abort: .word handle_fetch_abort
undefined: .word handle_undefined
