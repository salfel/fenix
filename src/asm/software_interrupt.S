.global setup_swi
.global handle_swi

.extern swi_handler
    
setup_swi:
    ldr r0, _swi_handler_addr
    ldr r1, _swi_handler
    str r1, [r0]

    bx lr

handle_swi:
    stmfd sp!, {r0-r12, lr}

    bl swi_handler

    cmp r0, #0x1
    beq keep_supervisor_mode

    ldmfd sp!, {r0-r12, pc}^

keep_supervisor_mode:
    msr spsr_c, #0x53

    ldmfd sp!, {r0-r12, pc}^

_swi_handler: .word handle_swi
_swi_handler_addr: .word 0x4030CE28
