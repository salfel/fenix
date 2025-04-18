.global switch_context
.global store_context
.global yield_task
.global should_switch

.equ SYSTEM_MODE, 0x5F
.equ SUPERVISOR_MODE, 0x53

switch_context:
    msr cpsr_c, #SYSTEM_MODE
    mov sp, r0

    blx r1

    msr cpsr_c, #SUPERVISOR_MODE

    bx lr

store_context:
    mrs r0, spsr
    str r0, temp_spsr

    mov r0, #0
    str r0, should_switch

    pop {{r0-r12, lr}}

    str lr, temp_pc

    msr cpsr_c, #SYSTEM_MODE
    push {{r0-r12, lr}}

    ldr r0, temp_spsr
    push {{r0}}

    mov r0, sp
    ldr r1, temp_pc

    msr cpsr_c, #SUPERVISOR_MODE

    b save_context

restore_context:
    msr cpsr_c, #SYSTEM_MODE
    mov sp, r0

    str r1, temp_pc

    pop {{r0}}
    msr cpsr, r0
    
    pop {{r0-r12, lr}}

    ldr pc, temp_pc

yield_task:
    stmfd sp!, {{r0-r12, lr}}

    mrs r3, cpsr
    push {{r3}}

    mov r2, r0

    mov r0, sp
    mov r1, lr

    msr cpsr_c, #SUPERVISOR_MODE

    b yield_context

temp_sp: .word 0

temp_spsr: .word 0
temp_pc: .word 0

should_switch: .word 0
