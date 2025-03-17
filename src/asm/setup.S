.section .init, "ax"

.extern stack_end
.extern irq_stack_end

setup:
    bl setup_modes
    bl setup_exceptions

    @ disable mmu
    mrc p15, 0, r1, c1, c0, 0
    bic r1, r1, #0x1
    mcr p15, 0, r1, c1, c0, 0

    @ Disable L1 Caches
    mrc p15, 0, r1, c1, c0, 0
    bic r1, r1, #(0x1 << 12) 
    bic r1, r1, #(0x1 << 2)  
    mcr p15, 0, r1, c1, c0, 0

    @ Invalidate Instruction cache
    MOV   r1, #0
    MCR   p15, 0, r1, c7, c5, 0

    @ Invalidate TLB
    mcr p15, 0, r1, c8, c7, 0
    
    b main

setup_modes:
    @ IRQ mode
    mov r0, #0xD2
    msr cpsr_c, r0
    ldr sp, =irq_stack_end
    
    @ Supervisor mode
    mov r0, #0xD3
    msr cpsr_c, r0
    ldr sp, =stack_end

    @ User/System mode
    mov r0, #0x1F
    msr cpsr_c, r0
    ldr sp, =user_stack_end

    @ Supervisor mode mode irq enabled
    mov r0, #0x53
    msr cpsr_c, r0

    bx lr
