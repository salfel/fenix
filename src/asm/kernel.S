.global switch_context
.global wait_store

.extern kernel_loop

switch_context:
    mov r2, #0x50
    msr cpsr_c, r2

    mov sp, r0
    blx r1

    svc #0x0

wait_store:
    stmfd sp!, {r0-r12, lr}

    mov r2, r0

    mrs r0, cpsr
    push {r0}

    mov r0, sp
    mov r1, lr
    mov r2, r2
    svc #0x1
