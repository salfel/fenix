.global handle_swi
.global handle_swi

.extern swi_handler

handle_swi:
    stmfd sp!, {r0-r12, lr}

    bl swi_handler

    cmp r0, #0x1
    beq keep_supervisor_mode

    ldmfd sp!, {r0-r12, pc}^

keep_supervisor_mode:
    msr spsr_c, #0x53

    ldmfd sp!, {r0-r12, pc}^

