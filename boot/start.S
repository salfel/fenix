.global start

.extern setup
.extern setup_irq
.extern setup_swi

start:
    ldr r0, base_data_abort_addr
    ldr r1, data_abort
    str r1, [r0]

    ldr r0, base_fetch_abort_addr
    ldr r1, fetch_abort
    str r1, [r0]

    @@ Setup sp in IRQ mode
    @mov r0, #0xD2
    @msr cpsr_c, r0
    @ldr r0, stack_base
    @ADD r0, #1024
    @mov sp, r0

    @@ Enter supervisor mode, irq disabled
    @mov r0, #0xD3
    @msr cpsr_c, r0
    @ldr r0, stack_base
    @ADD r0, #1024
    @ADD r0, #1024
    @mov sp, r0

    @@ Enter supervisor mode, irq enabled
    @mov r0, #0x53
    @msr cpsr_c, r0

    bl setup
    bl setup_irq

    bl main
    b hang

    
handle_data_abort:
    bl data_abort_handler

    b hang

handle_fetch_abort:
    bl fetch_abort_handler

    b hang

hang:
    b hang

stack_base: .word 0x4030BE30
base_data_abort_addr: .word 0x4030CE30
base_fetch_abort_addr: .word 0x4030CE2C
data_abort: .word handle_data_abort
fetch_abort: .word handle_fetch_abort
