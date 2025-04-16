.global switch_context
.global store_context
.global should_switch

switch_context:
    str sp, temp_sp
    mov sp, r0

    blx r1

    ldr sp, temp_sp

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

    ldr sp, temp_sp

    b save_context

restore_context:
    str sp, temp_sp
    mov sp, r0

    str r1, temp_pc

    pop {{r0}}
    msr cpsr, r0
    
    pop {{r0-r12, lr}}

    ldr pc, temp_pc

temp_sp: .word 0

temp_spsr: .word 0
temp_pc: .word 0

should_switch: .word 0
