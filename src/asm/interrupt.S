.global yield_task
.global restore_context
.global irq_handler

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

    mov r0, #0x1
    mov r1, sp
    ldr r2, next_pc
    mov r3, #0x0
    svc #0x0

    b kernel_loop

restore_context:
    msr cpsr_c, #0xDF
    str r1, next_pc

    mov sp, r0
    pop {r0}
    msr cpsr_xsfc, r0

    ldmfd sp!, {r0-r12, lr}

    ldr pc, next_pc

yield_task:
    mov r0, #0x1
    str r0, yielded

    bx lr

yielded: .word 0
next_pc: .word 0
temp_spsr: .word 0
