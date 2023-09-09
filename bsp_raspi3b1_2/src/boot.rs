use core::arch::global_asm;

#[no_mangle]
#[link_section = ".text._start_arguments"]
static BOOT_CORE_ID: u64 = 0;


global_asm!(r"
    .macro ADR_ABS register, symbol
        movz \register, #:abs_g2:\symbol
        movk \register, #:abs_g1_nc:\symbol
        movk \register, #:abs_g0_nc:\symbol
    .endm
");

global_asm!(r"
    .macro ADR_REL register, symbol
        adrp \register, \symbol
        add \register, \register, #:lo12:\symbol
    .endm
");

global_asm!(r"
    .section .text._start

    _start:
        // Only proceed on the boot core. Park it otherwise.
        mrs x0, MPIDR_EL1
        and x0, x0, 3                     // Get the Core ID
        ldr x1, BOOT_CORE_ID              // provided by bsp/__board_name__/cpu.rs
        cmp x0, x1
        b.ne .cpu_wait_loop

        // If execution reaches here, it is the boot core.
        // Initialize DRAM.
        ADR_ABS x0, __bss_start
        ADR_ABS x1, __bss_end_exclusive

    .L_bss_init_loop:
        cmp x0, x1
        b.eq .L_relocate_binary
        stp xzr, xzr, [x0], #16
        b .L_bss_init_loop

    .L_relocate_binary:
        ADR_REL x0, __binary_nonzero_start        // The address the binary got loaded to
        ADR_ABS x1, __binary_nonzero_start        // The address the binary was linked to.
        ADR_ABS x2, __binary_nonzero_end_exclusive

    .L_copy_loop:
        ldr x3, [x0], #8
        str x3, [x1], #8
        cmp x1, x2
        b.lo .L_copy_loop

    // Prepare the jump to Rust code.
        // Set the stack pointer.
        ADR_ABS x0, __boot_core_stack_end_exclusive
        mov sp, x0

        // Jump to the relocated Rust code.
        ADR_ABS x1, _start_rust
        br x1

    .cpu_wait_loop:
         wfe
         b .cpu_wait_loop

    .size _start, . - _start
    .type _start, function
    .global _start
"
);
