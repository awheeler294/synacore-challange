use std::fs;

use clap::Parser;

use machine::{Machine, OutputBuffer};

mod machine;
mod parse;

struct StandardOutputBuffer {
    buff: Vec<char>,
}

impl StandardOutputBuffer {
    fn new() -> Self {
        Self {
            buff: Vec::with_capacity(1024),
        }
    }
}

impl OutputBuffer for StandardOutputBuffer {
    fn push(&mut self, val: char) {
        self.buff.push(val);
    }

    fn flush(&mut self) {
        if self.buff.len() > 0 {
            print!("{}", self.buff.drain(0..).collect::<String>());
        }
    }

    fn len(&self) -> usize {
        self.buff.len()
    }

    fn contents(&self) -> &[char] {
        &self.buff[0..]
    }
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to program to run
    #[arg(short, long, default_value = "challenge.bin")]
    program: String,

    /// instead of running program print a decompiled version
    #[arg(short, long, default_value_t = false)]
    decompile: bool,
}

fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let args = Args::parse();

    let file_path = args.program;

    let file_contents = fs::read(&file_path)
        .expect(&format!("Could not read file {file_path}"))
        .chunks(2)
        .map(|chunk| (chunk[1] as u16) << 8 | chunk[0] as u16)
        .collect::<Vec<u16>>();

    // dbg!(&file_contents);
    //
    if args.decompile {
        println!("{}", parse::decompile(&file_contents));
    } else {
        let mut machine = Machine::new(file_contents, Box::new(StandardOutputBuffer::new()));
        machine.run();
        machine.flush_output_buffer();
    }
}
