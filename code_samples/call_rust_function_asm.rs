// Taken from https://web.mit.edu/rust-lang_v1.25/arch/amd64_ubuntu1404/share/doc/rust/html/unstable-book/language-features/global-asm.html

pub mod sally {
    use core::arch::{global_asm, asm};
    global_asm!(r#"
        .global foo
      foo:
        jmp baz
    "#);

    #[no_mangle]
    pub unsafe extern "C" fn baz() {
        println!("BAZ");
    }
}

// the symbols `foo` and `bar` are global, no matter where
// `global_asm!` was used.
extern "C" {
    fn foo();
    fn bar();
}

pub mod harry {
    use core::arch::{global_asm, asm};
    global_asm!(r#"
        .global bar
      bar:
        jmp quux
    "#);

    #[no_mangle]
    pub unsafe extern "C" fn quux() {
        println!("QUUX");
    }
}

fn main() {
    unsafe {
        foo();
        bar();
    };
    println!("OK");
}
