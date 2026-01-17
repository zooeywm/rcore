.altmacro

.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm

    .section .text
    .globl __alltraps
    .globl __restore
    # Align the addresses of __alltraps to 4 bytes, which is a requirement of the RISC-V privilege level specification
    .align 2

__alltraps:
    # exchange sp and sscratch
    csrrw sp, sscratch, sp
    # now sp->kernel stack, sscratch->user stack
    # allocate a TrapContext on kernel stack
    # 32 for general registers, 2 for sstatus and sepc
    addi sp, sp, -34*8
    # save general-purpose registers
    SAVE_GP 1
    # skip sp(x2), we will save it later
    SAVE_GP 3
    # skip tp(x4), application does not use it
    # save x5-x31
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr
    # we can use t0/t1/t2 freely, because they were saved on kernel stack
    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)
    # read user stack from sscratch and save it on the kernel stack
    csrr t2, sscratch
    sd t2, 2*8(sp)
    # set input argument of trap_handler(cx: &mut TrapContext)
    mv a0, sp
    call trap_handler


.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm

__restore:
    # case1: start running app by __restore
    # case2: back to U after handling trap
    mv sp, a0
    # now sp->kernel stack(after allocated), sscratch->user stack
    # restore sstatus/sepc
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    ld t2, 2*8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    csrw sscratch, t2
    # restore general registers except sp/tp
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    # release TrapContext on kernel stack
    addi sp, sp, 34*8
    csrrw sp,sscratch, sp
    # now sp->user stack, sscratch->kernel stack
    sret
