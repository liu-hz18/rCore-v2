# 操作系统启动时所需的指令以及字段
#
# 我们在 linker.ld 中将程序入口设置为了 _start，因此在这里我们将填充这个标签
# 它将会执行一些必要操作，然后跳转至我们用 rust 编写的入口函数
#
# 关于 RISC-V 下的汇编语言，可以参考 https://github.com/riscv/riscv-asm-manual/blob/master/riscv-asm.md
# %hi 表示取 [12,32) 位，%lo 表示取 [0,12) 位

    .section .text.entry
    .globl _start
# 目前 _start 的功能：将预留的栈空间写入 $sp，然后跳转至 rust_main
_start:
    # 计算 boot_page_table 的物理页号
    lui t0, %hi(boot_page_table)
    li t1, 0xffffffff00000000
    sub t0, t0, t1
    srli t0, t0, 12
    # 8 << 60 是 satp 中使用 Sv39 模式的记号
    li t1, (8 << 60)
    or t0, t0, t1
    # 写入 satp 并更新 TLB
    csrw satp, t0 # 页表启用了，之后就使用虚拟映射，但是此时需要照常执行pc, 所以需要一小段的恒等映射。
    sfence.vma

    # 加载栈地址
    lui sp, %hi(boot_stack_top)
    addi sp, sp, %lo(boot_stack_top) # 修改栈指针寄存器 sp 为 .bss.stack 段的结束地址, 由于栈是从高地址往低地址增长，所以高地址是初始的栈顶
    # 跳转至 rust_main
    lui t0, %hi(rust_main)
    addi t0, t0, %lo(rust_main)
    # 上面的 rust_main 实际上就是 0xffff_ffff_8020_0000，页表已经启用
    jr t0 # 我们的内核运行环境设置完成了，正式进入内核。
    jr x0 # 作为lab-1的暂时代码，移除rust_main中的panic!，使得程序返回到这里，之后跳转到0地址，引起LoadFault. 打印SUCCESS!

    # 回忆：bss 段是 ELF 文件中只记录长度，而全部初始化为 0 的一段内存空间
    # 这里声明字段 .bss.stack 作为操作系统启动时的栈
    .section .bss.stack
    .global boot_stack
boot_stack:
    # 16KB 启动栈大小
    .space 4096 * 16 # 2bit
    .global boot_stack_top
boot_stack_top:
    # 栈结尾

    # 初始内核映射所用的页表
    .section .data
    .align 12
boot_page_table:
    # boot_page_table是用二进制表示的根页表，其中包含两个 1GB 大页，
    # 分别是将虚拟地址 0x8000_0000 至 0xc000_0000 映射到物理地址 0x8000_0000 至 0xc000_0000，
    # 以及将虚拟地址 0xffff_ffff_8000_0000 至 0xffff_ffff_c000_0000 映射到物理地址 0x8000_0000 至 0xc000_0000。
    .quad 0
    .quad 0
    # 第 2 项：0x8000_0000 -> 0x8000_0000，0xcf 表示 VRWXAD 均为 1
    # 这里映射没有改变，因为在跳转到 rust_main 之前（即 jr t0）之前，PC 的值都还是 0x802xxxxx 这样的地址，即使是写入了 satp 寄存器，但是 PC 的地址不会变。
    # 为了执行这段中间的尴尬的代码，我们在页表里面也需要加入这段代码的地址的映射。
    # 跳转之后就没有问题了，因为 rust_main 这个符号本身是高虚拟地址（这点在 linker script 里面已经体现了）
    # （低地址的恒等映射）则保证程序替换页表后的短暂时间内，pc 仍然可以顺着低地址去执行内存中的指令。
    .quad (0x80000 << 10) | 0xcf
    .zero 507 * 8
    # 为了让程序能够正确跳转至高地址的 rust_main(0xffff_ffff_8020_0000 in linker.ld)，我们需要在 entry.asm 中先应用内核重映射，即将高地址映射到低地址。但我们不可能在替换页表的同时修改 pc，此时 pc 仍然处于低地址。
    # 这个页表只是启动时的一个简单页表，或者我们可以叫它“内核初始映射”
    # 1GB 的一个大页
    # 第 510 项：0xffff_ffff_8000_0000 -> 0x8000_0000，0xcf 表示 VRWXAD 均为 1
    # 510 的二进制是要索引虚拟地址的 VPN_3
    .quad (0x80000 << 10) | 0xcf
    .quad 0
