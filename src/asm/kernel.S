.global kernel_loop
.global user_mode

.extern kernel

kernel_loop:
    bl kernel

    ldr r0, =user_loop
    bl user_mode

    b kernel_loop

user_mode:
    push {lr}
    mov r1, #0x50
    msr cpsr_c, r1

    blx r0

    mov r0, #0x0
    svc #0x0

    pop {pc}
