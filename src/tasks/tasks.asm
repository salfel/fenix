.global switch_context
.global store_context
.global should_switch

switch_context:
    mov r1, sp
    mov sp, r0

    push {{r1, lr}}

    blx r1

    pop {{r1, lr}}
    bx lr

store_context:
    mrs r0, spsr
    str r0, temp_spsr

    mov r0, #0
    str r0, should_switch

    pop {{r0-r12, lr}}

    str lr, temp_pc

    msr cpsr_c, #0xD3
    push {{r0-r12, lr}}

    ldr r0, temp_spsr
    push {{r0}}

    mov r0, sp
    ldr r1, temp_pc

    b save_context

restore_context:
    mov sp, r0

    str r1, temp_pc

    pop {{r0}}
    msr cpsr, r0
    
    pop {{r0-r12, lr}}

    ldr pc, temp_pc

temp_spsr: .word 0
temp_pc: .word 0

should_switch: .word 0
