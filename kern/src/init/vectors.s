// TODO: make push and pop macros do two elements

.macro PUSH8 reg
    str \reg, [sp, #-8]!
.endm

.macro POP8 reg
    ldr \reg, [sp, #8]!
.endm

.macro PUSH16 reg
    str \reg, [sp, #-16]!
.endm

.macro POP16 reg
    ldr \reg, [sp, #16]!
.endm

.global context_save
context_save:
    PUSH8 x0
    PUSH8 x1
    PUSH8 x2
    PUSH8 x3
    PUSH8 x4
    PUSH8 x5
    PUSH8 x6
    PUSH8 x7
    PUSH8 x8
    PUSH8 x9
    PUSH8 x10
    PUSH8 x11
    PUSH8 x12
    PUSH8 x13
    PUSH8 x14
    PUSH8 x15
    PUSH8 x16
    PUSH8 x17
    PUSH8 x18
    PUSH8 x30

    PUSH16 q0
    PUSH16 q1
    PUSH16 q2
    PUSH16 q3
    PUSH16 q4
    PUSH16 q5
    PUSH16 q6
    PUSH16 q7
    PUSH16 q8
    PUSH16 q9
    PUSH16 q10
    PUSH16 q11
    PUSH16 q12
    PUSH16 q13
    PUSH16 q14
    PUSH16 q15
    PUSH16 q16
    PUSH16 q17
    PUSH16 q18
    PUSH16 q19
    PUSH16 q20
    PUSH16 q21
    PUSH16 q22
    PUSH16 q23
    PUSH16 q24
    PUSH16 q25
    PUSH16 q26
    PUSH16 q27
    PUSH16 q28
    PUSH16 q29
    PUSH16 q30
    PUSH16 q31

    bl handle_exception

    b context_restore


.global context_restore
context_restore:
    POP16 q31
    POP16 q30
    POP16 q29
    POP16 q28
    POP16 q27
    POP16 q26
    POP16 q25
    POP16 q24
    POP16 q23
    POP16 q22
    POP16 q21
    POP16 q20
    POP16 q19
    POP16 q18
    POP16 q17
    POP16 q16
    POP16 q15
    POP16 q14
    POP16 q13
    POP16 q12
    POP16 q11
    POP16 q10
    POP16 q9
    POP16 q8
    POP16 q7
    POP16 q6
    POP16 q5
    POP16 q4
    POP16 q3
    POP16 q2
    POP16 q1
    POP16 q0

    POP8 x30
    POP8 x18
    POP8 x17
    POP8 x16
    POP8 x15
    POP8 x14
    POP8 x13
    POP8 x12
    POP8 x11
    POP8 x10
    POP8 x9
    POP8 x8
    POP8 x7
    POP8 x6
    POP8 x5
    POP8 x4
    POP8 x3
    POP8 x2
    POP8 x1
    POP8 x0

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

