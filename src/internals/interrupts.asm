handle_interrupt:
    sub lr, lr, #4
    stmfd sp!, {{r0-r12, lr}}

    mrs r11, spsr
    push {{r11}}

    bl interrupt_handler

    dsb

    pop {{r11}}
    msr spsr, r11 

    ldmfd sp!, {{r0-r12, pc}}^
