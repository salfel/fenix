.global setup_irq

handle_irq:
    SUB lr, lr, #4
    STMFD sp!, {R0-R12, lr}

    MRS R11, SPSR
    PUSH {r11}

    bl handle_interrupt

    MOV R0, #0
    MCR P15, #0, R0, C7, C10, #4

    POP {r11}
    MSR SPSR, R11 

    LDMFD sp!, {R0-R12, pc}^

setup_irq:
    LDR r0, base_irq_addr
    LDR r1, irq_handler
    STR r1,[r0]

    BX lr

irq_handler: .word handle_irq
base_irq_addr: .word 0x4030CE38
