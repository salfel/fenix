    .global __wrap__start
    .type   __wrap__start, %function
__wrap__start:
    bl __real__start

    svc #0x0
