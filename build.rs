use std::path::PathBuf;

fn main() {
    let target_dir: PathBuf = match std::env::var("CARGO_TARGET_DIR") {
        Ok(dir) => PathBuf::from(dir),
        Err(_) => std::env::current_dir()
            .expect("Unable to get current dir")
            .join("target"),
    };
    let ld_file_path = target_dir.join("kernel.ld");
    std::fs::write(&ld_file_path, bsp_raspi3b1_2::LINKER_SCRIPT)
        .expect("Unable to write linker script to file");

    println!(
        "cargo:rustc-link-arg=--library-path={}",
        ld_file_path.canonicalize().unwrap().to_str().unwrap()
    );
    println!(
        "cargo:rustc-link-arg=--script={}",
        ld_file_path.canonicalize().unwrap().to_str().unwrap()
    );
}
