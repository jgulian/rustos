// TODO: make push and pop macros do two elements

.macro PUSH16 a, b
    stp  \b, \a, [SP, #-16]!
.endm

.macro POP16 a, b
    ldp  \a, \b, [SP], #16
.endm

.macro PUSH32 a, b
    stp  \b, \a, [SP, #-32]!
.endm

.macro POP32 a, b
    ldp  \a, \b, [SP], #32
.endm

.global context_save
context_save:
    PUSH16 x27, x26
    PUSH16 x25, x24
    PUSH16 x23, x22
    PUSH16 x21, x20
    PUSH16 x19, x18
    PUSH16 x17, x16
    PUSH16 x15, x14
    PUSH16 x13, x12
    PUSH16 x11, x10
    PUSH16 x9,  x8
    PUSH16 x7,  x6
    PUSH16 x5,  x4
    PUSH16 x3,  x2
    PUSH16 x1,  x0

    PUSH32 q31, q30
    PUSH32 q29, q28
    PUSH32 q27, q26
    PUSH32 q25, q24
    PUSH32 q23, q22
    PUSH32 q21, q20
    PUSH32 q19, q18
    PUSH32 q17, q16
    PUSH32 q15, q14
    PUSH32 q13, q12
    PUSH32 q11, q10
    PUSH32 q9,  q8
    PUSH32 q7,  q6
    PUSH32 q5,  q4
    PUSH32 q3,  q2
    PUSH32 q1,  q0

    mrs x0, TPIDR_EL0
    mrs x1, SP_EL0
    PUSH16 x0, x1
    mrs x0, SPSR_EL1
    mrs x1, ELR_EL1
    PUSH16 x0, x1

    mov x0, x29
    mrs x1, ESR_EL1
    mov x2, sp

    PUSH16 xzr, lr

    bl handle_exception

    POP16 lr, xzr

    b context_restore


.global context_restore
context_restore:
    POP16 x0, x1
    msr ELR_EL1, x0
    msr SPSR_EL1, x1
    mov x25, x0
    POP16 x0, x1
    msr SP_EL0, x0
    msr TPIDR_EL0, x1

    POP32 q0, q1
    POP32 q2, q3
    POP32 q4, q5
    POP32 q6, q7
    POP32 q8, q9
    POP32 q10, q11
    POP32 q12, q13
    POP32 q14, q15
    POP32 q16, q17
    POP32 q18, q19
    POP32 q20, q21
    POP32 q22, q23
    POP32 q24, q25
    POP32 q26, q27
    POP32 q28, q29
    POP32 q30, q31

    POP16 x0, x1
    POP16 x2, x3
    POP16 x4, x5
    POP16 x6, x7
    POP16 x8, x9
    POP16 x10, x11
    POP16 x12, x13
    POP16 x14, x15
    POP16 x16, x17
    POP16 x18, x19
    POP16 x20, x21
    POP16 x22, x23
    POP16 x24, x25
    POP16 x26, x27

    ret

.macro HANDLER source, kind
    .align 7
    stp     lr, xzr, [SP, #-16]!
    stp     x28, x29, [SP, #-16]!

    mov     x29, \source
    movk    x29, \kind, LSL #16
    bl      context_save

    ldp     x28, x29, [SP], #16
    ldp     lr, xzr, [SP], #16
    eret
.endm

.align 11
.global vectors
vectors:
    HANDLER 0, 0
    HANDLER 0, 1
    HANDLER 0, 2
    HANDLER 0, 3

    HANDLER 1, 0
    HANDLER 1, 1
    HANDLER 1, 2
    HANDLER 1, 3

    HANDLER 2, 0
    HANDLER 2, 1
    HANDLER 2, 2
    HANDLER 2, 3

    HANDLER 3, 0
    HANDLER 3, 1
    HANDLER 3, 2
    HANDLER 3, 3

