use std::path::PathBuf;
use std::borrow::Cow;
use clap::{Parser, arg};

#[derive(Parser)]
pub struct Args {
    #[arg(short, long)]
    serial_port: String,
}

fn main() {
    let args = Args::parse();
    let port = serialport::new(args.serial_port, 921600);
}
