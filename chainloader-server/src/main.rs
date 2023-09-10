use clap::{arg, Parser};
use serialport::SerialPort;
use std::io::Read;
use std::print;
use std::{fs::File, path::PathBuf};

#[derive(Parser)]
pub struct Args {
    #[arg(short, long)]
    serial_port: String,

    #[arg(short, long)]
    kernel_fpath: PathBuf,
}

fn send_size(serial: &mut Box<dyn SerialPort>, size: u32) {
    let b = [
        size as u8,
        (size >> 8) as u8,
        (size >> 16) as u8,
        (size >> 24) as u8,
    ];
    serial.write(&b).expect("Unable to write kernel file size");
}

fn start(args: &Args, mut serial: Box<dyn SerialPort>) {
    let mut buff = [0; 3];
    while buff != [3; 3] {
        let _ = serial
            .read(&mut buff);
        println!("{buff:?}");
        std::thread::sleep(std::time::Duration::from_secs(1));

    }
    println!("Got init from target");

    let mut kernel_file = File::open(&args.kernel_fpath).expect("Unable to load kernel file");
    let kernel_size: u32 = kernel_file
        .metadata()
        .expect("Unable to get metadata")
        .len()
        .try_into()
        .expect("File size > u32::MAX");
    println!("Size of kernel: {kernel_size}");
    send_size(&mut serial, kernel_size);
    println!("Size sent to target");

    let mut rep = [0u8; 2];
    let _ = serial.read(&mut rep);
    if rep != [79, 75] {
        println!("Expected OK from target, got {rep:?}");
        println!("Aborting");
        return;
    }

    println!("Loading the kernel to the target ...");
    let nbatches = (kernel_size / 512) as usize;
    let mut kbuff = [0; 512];
    let mut b = 0;
    let mut r = 0;
    let mut w = 0;
    while w < kernel_size {
        print!("\r[{}{}]", "=".repeat(b), " ".repeat(nbatches - b));
        match kernel_file.read(&mut kbuff) {
            Ok(n) => {
                let n: u32 = n.try_into().unwrap();
                r += n;
            }
            Err(e) => {
                print!("\n");
                println!("Error while reading kernel file to buffer");
                println!("{e:?}");
                println!("Aborting");
                return;
            }
        }

        match serial.write(&kbuff) {
            Ok(n) => {
                let n: u32 = n.try_into().unwrap();
                w += n;
            }
            Err(e) => {
                print!("\n");
                println!("Error while writing kernel binary to serial");
                println!("{e:?}");
                println!("Aborting");
                return;
            }
        }
        assert_eq!(r, w, "\nRead and Write counters are different");
        b += 1;
    }
    println!("Loading complete");
    println!("");
}

fn main() {
    let args = Args::parse();
    let port = serialport::new(&args.serial_port, 921600);
    println!("Waiting for {} to be available ...", args.serial_port);
    loop {
        match port.clone().open() {
            Ok(s) => {
                println!("Got serial connection !");
                start(&args, s)
            }
            Err(serialport::Error {
                kind: serialport::ErrorKind::Io(std::io::ErrorKind::NotFound),
                ..
            }) => {
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
            Err(e) => {
                panic!("Unexpected error occured: {e:?}");
            }
        }
    }
}
