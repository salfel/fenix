.global irq_handler
.global yield_task
.global restore_context

irq_handler:
    sub lr, lr, #4
    stmfd sp!, {r0-r12, lr}

    mrs r11, spsr
    push {r11}

    bl handle_interrupt

    mov r0, #0
    mcr p15, #0, r0, c7, c10, #4

    pop {r11}
    msr spsr, r11 
    and r11, r11, #0b11111
    cmp r11, #0b10011
    beq return_interrupt

    ldr r0, yielded
    cmp r0, #0x1
    beq store_context

return_interrupt:
    ldmfd sp!, {r0-r12, pc}^

store_context:
    ldmfd sp!, {r0-r12, lr}

    str lr, next_pc

    push {r0}
    mrs r0, spsr
    str r0, temp_spsr
    pop {r0}

    msr cpsr_c, #0xDF
    stmfd sp!, {r0-r12, lr}

    mov r0, #0x0
    str r0, yielded

    ldr r0, temp_spsr
    push {r0}

    mov r0, sp
    ldr r1, next_pc
    mov r2, #0x0
    svc #0x1

restore_context:
    ldmfd r0!, {r2}
    msr spsr, r2

    push {r1}

    msr cpsr_c, #0xDF
    mov sp, r0
    pop {r0-r12, lr}
    msr cpsr_c, #0xD3

    ldmfd sp!, {pc}^

yield_task:
    mov r0, #0x1
    str r0, yielded

    bx lr

yielded: .word 0
next_pc: .word 0
temp_spsr: .word 0
