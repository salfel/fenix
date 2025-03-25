.global setup_mmu

setup_mmu:
    @ disable mmu
    mrc p15, 0, r1, c1, c0, 0
    bic r1, r1, #0x1
    mcr p15, 0, r1, c1, c0, 0

    @ disable L1 Caches
    mrc p15, 0, r1, c1, c0, 0
    bic r1, r1, #(0x1 << 12) 
    bic r1, r1, #(0x1 << 2)  
    mcr p15, 0, r1, c1, c0, 0

    @ invalidate l1 caches
    mov r1, #0
    mcr p15, 0, r1, c7, c5, 0

    @ invalidate data caches
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

    @ invalidate tlb
    mcr p15, 0, r1, c8, c7, 0

    @ branch prediction enable
    mov r1, #0
    mrc p15, 0, r1, c1, c0, 0
    orr r1, r1, #(0x1 << 11)
    mcr p15, 0, r1, c1, c0, 0

    @ enable D-side prefetch
    mrc p15, 0, r1, c1, c0, 1
    orr r1, r1, #(0x1 <<2)
    mcr p15, 0, r1, c1, c0, 1
    dsb
    isb

    ldr r0, =0b00000000000000000000110111100010
    ldr r1, =level1_page_table
    ldr r3, =4095

write_pte:
    orr r2, r0, r3, lsl #20
    str r2,  [r1, r3, lsl #2]
    subs r3, r3, #1
    bne write_pte

    @ initialize mmu
    mov r1, #0x0
    mcr p15, 0, r1, c2, c0, 2
    ldr r1,  =level1_page_table
    mcr p15, 0, r1, c2, c0, 0

    @ set all domains to client
    ldr r1, =0x55555555
    mcr p15, 0, r1, c3, c0, 0

    @ enable mmu
    mrc p15, 0, r1, c1, c0, 0
    orr r1, r1, #0x1
    mcr p15, 0, r1, c1, c0, 0

    bx lr
