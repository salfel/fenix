.global kernel_loop
.global user_mode
.global switch_context

.extern kernel

kernel_loop:
    bl kernel

    b kernel_loop

switch_context:
    mov r2, #0x50
    msr cpsr_c, r2

    mov sp, r0
    blx r1

    mov r0, #0x0
    svc #0x0
    
    b kernel_loop

user_mode:
    push {lr}
    mov r1, #0x50
    msr cpsr_c, r1

    blx r0

    mov r0, #0x0
    svc #0x0

    pop {pc}
