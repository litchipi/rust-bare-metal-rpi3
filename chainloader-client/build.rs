use std::path::PathBuf;

fn main() {
    let ld_file_path = std::env::var("LD_FILE_PATH")
        .expect("Please provide linker script path to the env var LD_FILE_PATH");
    let ld_file_path = PathBuf::from(ld_file_path);

    println!(
        "cargo:rustc-link-arg=--library-path={}",
        ld_file_path.canonicalize().unwrap().to_str().unwrap()
    );
    println!(
        "cargo:rustc-link-arg=--script={}",
        ld_file_path.canonicalize().unwrap().to_str().unwrap()
    );
}
