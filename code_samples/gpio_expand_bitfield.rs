use ::tock_registers::fields::Field;
use ::tock_registers::fields::{TryFromValue, FieldValue};

mod GPFSEL1 {
    #[derive(Clone, Copy)]
    pub struct Register;

    // Offset: 15     numbits 3
    //    new(mask, shift)
    //    A: 1 << 2 = 0000_0100
    //    B: (1 << 2) - 1 = 0000_0011
    //    A + B = 0000_0111
    pub const FSEL15: Field<u32, Register> = Field::<u32, Register>::new((1 << (3 - 1)) + ((1 << (3 - 1)) - 1), 15);

    pub mod FSEL15 {
        use super::Register;
        pub const Input: FieldValue<u32, Register> = FieldValue::<u32, Register>::new((1 << (3 - 1)) + ((1 << (3 - 1)) - 1), 15, 0b000);
        pub const Output: FieldValue<u32, Register> = FieldValue::<u32, Register>::new((1 << (3 - 1)) + ((1 << (3 - 1)) - 1), 15, 0b001);
        pub const AltFunc0: FieldValue<u32, Register> = FieldValue::<u32, Register>::new((1 << (3 - 1)) + ((1 << (3 - 1)) - 1), 15, 0b100);

        pub const SET: FieldValue<u32, Register> = FieldValue::<u32, Register>::new((1 << (3 - 1)) + ((1 << (3 - 1)) - 1), 15, (1 << (3 - 1)) + ((1 << (3 - 1)) - 1));
        pub const CLEAR: FieldValue<u32, Register> = FieldValue::<u32, Register>::new((1 << (3 - 1)) + ((1 << (3 - 1)) - 1), 15, 0);

        #[repr(u32)]
        #[derive(Copy, Clone, PartialEq, Eq)]
        pub enum Value {
            Input = 0b000,
            Output = 0b001,
            AltFunc0 = 0b100,
        }

        impl TryFromValue<u32> for Value {
            // ...
        }

        impl From<Value> for FieldValue<u32, Register> {
            fn from(v: Value) -> Self {
                Self::new((1 << (3 - 1)) + ((1 << (3 - 1)) - 1), 15, v as u32)
            }
        }
    }
}
