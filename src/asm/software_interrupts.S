.global handle_swi
.global handle_swi

.extern swi_handler

handle_swi:
    stmfd sp!, {r0-r12, lr}

    sub lr, lr, #4
    ldr r12, [lr]
    bic r12, r12, #0xFF000000

    push {r0-r3, r12}
    mov r1, sp

    sub sp, sp, #8
    mov r0, sp

    bl swi_handler

    ldr r0, [sp, #4]
    ldrb r1, [sp]
    add sp, #8
    
    add sp, sp, #20

    cmp r1, #0x1
    beq exit

    ldmfd sp!, {r0-r12, pc}^

exit:
    msr spsr_c, #0x53

    ldr r1, =kernel_loop
    str r1, [sp, #52]

    ldmfd sp!, {r0-r12, pc}^

