.global setup_swi
.global handle_swi
.global swi
.global user_mode

.extern main_loop
.extern swi
.extern user_stack_end

user_mode:
    mov r0, #0x50
    msr cpsr_c, r0

    b main_loop
    
swi:
    svc #0x0

    bx lr

setup_swi:
    ldr r0, _swi_handler_addr
    ldr r1, _swi_handler
    str r1, [r0]

    bx lr

handle_swi:
    movs pc, lr

_swi_handler: .word handle_swi
_swi_handler_addr: .word 0x4030CE28
