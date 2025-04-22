.global setup_modes
.global setup_caches

setup_modes:
    @ IRQ mode
    mov r0, #0xD2
    msr cpsr_c, r0
    ldr sp, =irq_stack_end
    
    @ Supervisor mode
    mov r0, #0xD3
    msr cpsr_c, r0
    ldr sp, =stack_end

    @ Enable IRQ
    mov r0, #0x53
    msr cpsr_c, r0

    bx lr

setup_caches:
    push {lr}

    bl disable_l1_caches
    bl invalidate_l1_caches
    bl invalidate_data_caches
    bl branch_prediction_enable
    bl enable_d_side_prefetch

    pop {pc}

disable_l1_caches:
    mrc p15, 0, r1, c1, c0, 0
    bic r1, r1, #(0x1 << 12) 
    bic r1, r1, #(0x1 << 2)  
    mcr p15, 0, r1, c1, c0, 0

    bx lr

invalidate_l1_caches:
    mov r1, #0
    mcr p15, 0, r1, c7, c5, 0

    bx lr

invalidate_data_caches:
    mrc   p15, 1, r0, c0, c0, 0
    ldr r3, =0x1ff
    and   r0, r3, r0, lsr #13

    mov   r1, #0
way_loop:
    mov   r3, #0

set_loop:
    mov   r2, r1, lsl #30
    orr   r2, r3, lsl #5
    mcr   p15, 0, r2, c7, c6, 2
    add   r3, r3, #1
    cmp   r0, r3
    bgt   set_loop
    add   r1, r1, #1
    cmp   r1, #4
    bne   way_loop

    bx lr

invalidate_tlb:
    mcr p15, 0, r1, c8, c7, 0

    bx lr

branch_prediction_enable:
    mov r1, #0
    mrc p15, 0, r1, c1, c0, 0
    orr r1, r1, #(0x1 << 11)
    mcr p15, 0, r1, c1, c0, 0

    bx lr

enable_d_side_prefetch:
    mrc p15, 0, r1, c1, c0, 1
    orr r1, r1, #(0x1 <<2)
    mcr p15, 0, r1, c1, c0, 1
    dsb
    isb

    bx lr

write_pte:
    orr r2, r0, r3, lsl #20
    str r2,  [r1, r3, lsl #2]
    subs r3, r3, #1
    bge write_pte

    bx lr

