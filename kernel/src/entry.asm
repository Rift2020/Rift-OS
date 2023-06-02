    .section .text.entry
    .globl _start
_start:
	# 保存cpu hartid到tp
	mv		tp, a0 
	# t0=启动页表虚拟地址 (t0=(boot_page_table_sv39>>12)<<12);(使尾12位为0)
    lui     t0, %hi(boot_page_table_sv39)
	# t1=虚实偏移量，和常量中的PHYS_VIRT_OFFSET是同一个值
    li      t1, 0xffffffffc0000000-0x80000000
	# t0=启动页表物理地址
    sub     t0, t0, t1
	# t0=t0>>12 由PA变为PPN
    srli    t0, t0, 12

	# 下三行：使得satp=(8<<60)|t0
    li      t1, 8 << 60
    or      t0, t0, t1
    csrw    satp, t0
    
	#刷新TLB
	sfence.vma

	# sp设为bootstacktop虚拟地址
    lui sp, %hi(bootstacktop)
	
	# 跳转到rust_main
    lui t0, %hi(rust_main)
    addi t0, t0, %lo(rust_main)
    jr t0

	.section .text.entry
    .globl _start2
_start2:
	mv		tp, a0 
	mv      sp, a1
    lui     t0, %hi(boot_page_table_sv39)
    li      t1, 0xffffffffc0000000-0x80000000
    sub     t0, t0, t1
    srli    t0, t0, 12

    li      t1, 8 << 60
    or      t0, t0, t1
    csrw    satp, t0
    
	sfence.vma

	
    lui t0, %hi(rust_main)
    addi t0, t0, %lo(rust_main)
    jr t0

    .section .bss.stack
    .align 12
    .global bootstack
boot_stack_lower_bound:
    .space 4096 * 64 #256KB stack
    .global bootstacktop
bootstacktop:

    .section .data
    .align 12   # page align
boot_page_table_sv39:
    # 0x8000_0000 -> 0x8000_0000 (1G) 防止上面汇编，刷完TLB后无法继续执行剩下四条(内核初始化后不再保留)
	# 下面两个在内核初始化时会精细化的重新映射
	# 0xffff_ffc8_0000_0000 -> 0x0 (3G) 线性偏移映射方便内核访问整个物理空间
	# 0xffff_ffff_c000_0000 -> 0x8000_0000 (1G) 内核虚拟空间映射到物理空间
    .zero 8 * 2
	.quad (0x80000 << 10) | 0xcf # VRWXAD
	.zero 8 * 253
	.zero 8 * 32
	.quad (0x0 << 10) | 0xcf # VRWXAD
	.quad (0x40000 << 10) |0xcf 
	.quad (0x80000 <<10) |0xcf
	.zero 8 * 29
	.zero 8 * 191
	.quad (0x80000 << 10) | 0xcf # VRWXAD
