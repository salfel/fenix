.global __aeabi_unwind_cpp_pr0

.global setup_exceptions
.global vectors

.align 5
vectors:
    b _start
    b undefined_handler
    b handle_swi
    b fetch_abort_handler
    b data_abort_handler
    b .
    b irq_handler
    b .

setup_exceptions:
    ldr r0, =vectors
    mcr p15, 0, r0, c12, c0, 0
    dsb

    bx lr


__aeabi_unwind_cpp_pr0:
    b .
