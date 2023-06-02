/*
 * 重排elf文件使得低地址到高地址分别为text,rodata,data,bss段
 * stext代表text段起始地址，etext为text段结束地址，其他类似
*/
OUTPUT_ARCH(riscv)
ENTRY(_start)
BASE_ADDRESS = 0xffffffffc0200000;

ENTRY_ADDRESS = 0x80200000;

SECTIONS
{
    . = BASE_ADDRESS;
    skernel = .;

    .text ALIGN(4K): AT(ENTRY_ADDRESS) {
    stext = .;
        *(.text.entry)
        *(.text .text.*)
    }

    . = ALIGN(4K);
    etext = .;
    srodata = .;
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }

    . = ALIGN(4K);
    erodata = .;
    sdata = .;
    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }

    . = ALIGN(4K);
    edata = .;

	sbss_with_stack = .;
    .bss : {
        *(.bss.stack)
        sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    }

    . = ALIGN(4K);
    ebss = .;
    ekernel = .;

    /DISCARD/ : {
        *(.eh_frame)
    }

}
