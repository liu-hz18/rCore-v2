# 我们将会用一个宏来用循环保存寄存器。这是必要的设置
.altmacro
# 寄存器宽度对应的字节数
.set    REG_SIZE, 8
# Context 的大小
.set    CONTEXT_SIZE, 34

# 宏：将寄存器存到栈上
.macro SAVE reg, offset
    sd  \reg, \offset*REG_SIZE(sp) # 8Byte = 32bit
.endm

.macro SAVE_N n
    SAVE  x\n, \n
.endm


# 宏：将寄存器从栈中取出
.macro LOAD reg, offset
    ld  \reg, \offset*REG_SIZE(sp)
.endm

.macro LOAD_N n
    LOAD  x\n, \n
.endm

    .section .text
    .globl __interrupt
# 进入中断
# 保存 Context 并且进入 Rust 中的中断处理函数 interrupt::handler::handle_interrupt()
__interrupt:
    # 因为线程当前的栈不一定可用，必须切换到内核栈来保存 Context 并进行中断流程
    # 因此，我们使用 sscratch 寄存器保存内核栈地址
    # 思考：sscratch 的值最初是在什么地方写入的？
    
    # 交换 sp 和 sscratch（切换到内核栈）
    csrrw   sp, sscratch, sp

    # 在栈上开辟 Context 所需的空间
    addi    sp, sp, -CONTEXT_SIZE*REG_SIZE

    # 保存通用寄存器，除了 x0（固定为 0）
    SAVE    x1, 1
    # 将本来的栈地址 sp（即 x2）保存
    csrr    x1, sscratch
    SAVE    x1, 2
    # 保存 x3 至 x31
    .set    n, 3
    .rept   29
        SAVE_N  %n
        .set    n, n + 1
    .endr

    # 取出 CSR 并保存
    csrr    s1, sstatus
    csrr    s2, sepc
    SAVE    s1, 32
    SAVE    s2, 33

    # 调用 handle_interrupt, 传入参数, 通过汇编实现
    # context: &mut Context
    mv      a0, sp
    # scause: Scause
    csrr    a1, scause # scause和stval作为临时变量，而不是上下文
    # stval: usize
    csrr    a2, stval
    jal  handle_interrupt

    .globl __restore
# 离开中断, 在handle_interrupt之后执行
# 此时内核栈顶被推入了一个 Context，而 a0 指向它
# 接下来从 Context 中恢复所有寄存器，并将 Context 出栈（用 sscratch 记录内核栈地址）
# 最后跳转至 Context 中 sepc 的位置
__restore:
    # a0 应指向被压在内核栈中的 Context
    # 从 a0 中读取 sp
    # 思考：a0 是在哪里被赋值的？（有两种情况）
    # __restore 现在会将 a0 寄存器视为一个 *mut Context 来读取，因此我们在执行第一个线程时只需调用 __restore(context)
    # 如果是程序发生了中断，执行到 __restore 的时候，a0 的值又是谁赋予的呢？
    # 当发生中断时，在 __restore 时，a0 寄存器的值是 handle_interrupt 函数的返回值。
    # 也就是说，如果我们令 handle_interrupt 函数返回另一个线程的 *mut Context，就可以在时钟中断后跳转到这个线程来执行。
    mv sp, a0 # 让其从 a0 中读取我们设计好的 Context, 我们可以直接在 Rust 代码中调用 __restore(context)
    # 恢复 CSR
    LOAD    s1, 32
    LOAD    s2, 33
    csrw    sstatus, s1
    csrw    sepc, s2

    # 将内核栈地址写入 sscratch
    addi    t0, sp, CONTEXT_SIZE * REG_SIZE
    csrw    sscratch, t0

    # 恢复通用寄存器
    LOAD    x1, 1
    # 恢复 x3 至 x31
    .set    n, 3
    .rept   29
        LOAD_N  %n
        .set    n, n + 1
    .endr

    # 恢复 sp（又名 x2）这里最后恢复是为了上面可以正常使用 LOAD 宏
    LOAD    x2, 2
    sret
