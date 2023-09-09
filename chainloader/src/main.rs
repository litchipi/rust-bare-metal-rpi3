use std::path::PathBuf;
use std::borrow::Cow;
use clap::{Parser, arg};

#[derive(Parser)]
pub struct Args {
    #[arg(short, long)]
    serial_port: PathBuf,
}

fn main() {
    let args = Args::parse();
    let port = serialport::new(Cow::new(args.serial_port), 921600);
}
