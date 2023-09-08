use tock_registers::{register_bitfields, register_structs, registers::ReadWrite};
use tock_registers::fields::Field;

mod gpio {
    
    #[repr(C)]
    struct RegisterBlock {
        _reserved1: [u8; 0x04 - 0x00],
        GPFSEL1: ReadWrite<u32, GPFSEL1::Register>,
        _reserved2: [u8; 0x94 - 0x08],
    }

    const _: () = {
        const SUM_MAX_ALIGN: (usize, usize) = {
            const SUM_MAX_ALIGN: (usize, usize) = {
                const SUM_MAX_ALIGN: (usize, usize) = {
                    const SUM_MAX_ALIGN: (usize, usize) = (0, 0);
                    const SUM: usize = SUM_MAX_ALIGN.0;
                    const MAX_ALIGN: usize = SUM_MAX_ALIGN.1;
                    if !(SUM == 0x00) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "Invalid start offset for padding _reserved1 (expected 0 but actual value differs)",
                                ),
                            );
                        }
                    }
                    (0x04, MAX_ALIGN)
                };
                const SUM: usize = SUM_MAX_ALIGN.0;
                const MAX_ALIGN: usize = SUM_MAX_ALIGN.1;
                if !(SUM == 0x04) {
                    {
                        ::core::panicking::panic_display(
                            &"Invalid start offset for field GPFSEL1 (expected 4 but actual value differs)",
                        );
                    }
                }
                const ALIGN: usize = core::mem::align_of::<
                    ReadWrite<u32, GPFSEL1::Register>,
                >();
                #[allow(clippy::bad_bit_mask)]
                {
                    if !(SUM & (ALIGN - 1) == 0) {
                        {
                            ::core::panicking::panic_display(
                                &"Invalid alignment for field GPFSEL1 (offset differs from expected)",
                            );
                        }
                    }
                }
                const NEW_SUM: usize = SUM
                    + core::mem::size_of::<ReadWrite<u32, GPFSEL1::Register>>();
                if !(NEW_SUM == 0x08) {
                    {
                        ::core::panicking::panic_display(
                            &"Invalid end offset for field GPFSEL1 (expected 8 but actual value differs)",
                        );
                    }
                }
                const NEW_MAX_ALIGN: usize = if ALIGN > MAX_ALIGN {
                    ALIGN
                } else {
                    MAX_ALIGN
                };
                (NEW_SUM, NEW_MAX_ALIGN)
            };
            const SUM: usize = SUM_MAX_ALIGN.0;
            const MAX_ALIGN: usize = SUM_MAX_ALIGN.1;
            if !(SUM == 0x08) {
                {
                    ::core::panicking::panic_fmt(
                        format_args!(
                            "Invalid start offset for padding _reserved2 (expected 8 but actual value differs)",
                        ),
                    );
                }
            }
            (0x94, MAX_ALIGN)
        };
        const SUM: usize = SUM_MAX_ALIGN.0;
        const MAX_ALIGN: usize = SUM_MAX_ALIGN.1;
        if !(SUM == 0x94) {
            ::core::panicking::panic("assertion failed: SUM == 0x94")
        }
        const STRUCT_SIZE: usize = core::mem::size_of::<RegisterBlock>();
        const ALIGNMENT_CORRECTED_SIZE: usize = if 0x94 % MAX_ALIGN != 0 {
            0x94 + (MAX_ALIGN - (0x94 % MAX_ALIGN))
        } else {
            0x94
        };
        if !(STRUCT_SIZE == ALIGNMENT_CORRECTED_SIZE) {
            {
                ::core::panicking::panic_display(
                    &"Invalid size for struct RegisterBlock (expected 148, actual struct size differs)",
                );
            }
        }
    };
}
