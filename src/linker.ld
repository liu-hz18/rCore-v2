/* 链接脚本，调整内存布局 */
/* 有关 Linker Script 可以参考：https://sourceware.org/binutils/docs/ld/Scripts.html */
/* see https://rcore-os.github.io/rCore-Tutorial-deploy/docs/lab-0/guide/part-6.html */

/* 目标架构 */
OUTPUT_ARCH(riscv)

/* 执行入口 */
ENTRY(_start)

/* 数据存放起始地址 */
/* OpenSBI 将自身放在 0x80000000，完成初始化后会跳转到 0x80200000，因此 _start 必须位于这个地址 */
BASE_ADDRESS = 0xffffffff80200000; /* 修改为虚拟地址 */
/* 操作系统执行文件会认为所有的符号都是在这个高地址上的 */
/* 但是 我们在硬件上只能将内核加载到 0x8020_0000 开始的内存空间上，此时的 pc 也会调转到这里。 */

/* 我们可以用 *( ) 来表示 将各个文件中所有 符合括号内要求 的输入段 放在当前的位置。 */
/* 把每个数据段都对齐到了 4KB，一个 4KB 的虚拟页中不会包含两个段，这意味着这个页的属性是可以确定的，标注属性更加方便 */
SECTIONS
{
    /* . 表示当前地址（location counter） */
    . = BASE_ADDRESS;

    /* start 符号表示全部的开始位置 */
    /* 单独的一个 . 为当前地址（Location Counter） */
    kernel_start = .;

    /* 加入对齐 */
    . = ALIGN(4K);
    text_start = .;

    /* .text 字段 */
    .text : {
        /* 把 entry 函数放在最前面 */
        *(.text.entry)
        /* 要链接的文件的 .text 字段集中放在这里 */
        *(.text .text.*)
    }

    /* 加入对齐 */
    . = ALIGN(4K);
    rodata_start = .;

    /* .rodata 字段 */
    .rodata : {
        /* 要链接的文件的 .rodata 字段集中放在这里 */
        *(.rodata .rodata.*)
    }
    
    /* 加入对齐 */
    . = ALIGN(4K);
    data_start = .;

    /* .data 字段 */
    .data : {
        /* 要链接的文件的 .data 字段集中放在这里 */
        *(.data .data.*)
    }

    /* 加入对齐 */
    . = ALIGN(4K);
    bss_start = .;

    /* .bss 字段 */
    /* .bss 字段一般包含全局变量的名称和长度，在执行时由操作系统分配空间并初始化为零。*/
    /* 由于我们编写操作系统，自然就无法向操作系统索要空间。但是此时我们具有随意使用内存空间的权力，因此我们可以在内存中随意划一段空间，然后用相应的算法来实现一个堆。  */
    .bss : {
        /* 要链接的文件的 .bss 字段集中放在这里 */
        *(.sbss .bss .bss.*)
    }

    /* 结束地址 */
    /* 加入对齐 */
    . = ALIGN(4K);
    kernel_end = .;
}
