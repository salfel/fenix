.global setup_modes

setup_modes:
    @ IRQ mode
    mov r0, #0xD2
    msr cpsr_c, r0
    ldr sp, =irq_stack_end
    
    @ Supervisor mode
    mov r0, #0xD3
    msr cpsr_c, r0
    ldr sp, =stack_end

    @ User/System mode
    mov r0, #0x1F
    msr cpsr_c, r0
    ldr sp, =user_stack_end

    @ Supervisor mode mode irq enabled
    mov r0, #0x53
    msr cpsr_c, r0

    bx lr
