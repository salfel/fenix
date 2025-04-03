.global switch_context
.global wait_store

.extern kernel_loop

switch_context:
    mov r2, #0x50
    msr cpsr_c, r2

    mov sp, r0
    blx r1

    svc #0x0
