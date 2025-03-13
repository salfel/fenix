.global switch_context
.global wait_store

.extern kernel_loop

switch_context:
    mov r2, #0x50
    msr cpsr_c, r2

    mov sp, r0
    blx r1

    mov r0, #0x0
    svc #0x0
    
    b kernel_loop

wait_store:
    stmfd sp!, {r0-r12, lr}

    mov r3, r0

    mrs r0, cpsr
    push {r0}

    mov r0, #0x1
    mov r1, sp
    mov r2, lr
    mov r3, r3
    svc #0x0

    b kernel_loop
