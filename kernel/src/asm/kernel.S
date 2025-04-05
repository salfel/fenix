.global switch_context

.extern kernel_loop

switch_context:
    msr cpsr_c, #0xDF
    mov sp, r0
    msr cpsr_c, #0xD3

    mov r2, #0x50
    msr spsr_c, r2

    stmfd sp!, {r1}
    ldmfd sp!, {pc}^
