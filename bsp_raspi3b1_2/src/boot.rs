use core::arch::global_asm;

global_asm!("
    .section .text._start
    _start:
    .core_wait:
    	wfe
    	b .core_wait

    .size _start, . - _start
    .type _start, function
    .global _start
");
