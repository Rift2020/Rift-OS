	.section .text.entry
	.global _start
_start:
    la sp, boot_stack_top  #设置sp指向栈底(sp是指向栈顶的指针，但此时栈空)
	call rust_main

    .section .bss.stack
    .globl boot_stack_lower_bound
boot_stack_lower_bound:
    .space 4096 * 16  #64KB stack
	.globl boot_stack_top
boot_stack_top:
