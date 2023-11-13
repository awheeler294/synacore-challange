use std::{
    fs,
    io::{self, BufRead},
};

use clap::Parser;

use log::{debug, error};
use machine::{Machine, RunState};
use parse::parse_16_bit_little_endian;

mod machine;
mod parse;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to program to run
    #[arg(short, long, default_value = "challenge.bin")]
    program: String,

    /// Instead of running program print a decompiled version
    #[arg(short, long, default_value_t = false)]
    decompile: bool,
}

fn main() {
    // simple_logger::init_with_level(log::Level::Debug).unwrap();
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let args = Args::parse();

    let file_path = args.program;

    let file_contents = fs::read(&file_path).expect(&format!("Could not read file {file_path}"));

    let program = parse_16_bit_little_endian(&file_contents);

    // dbg!(&file_contents);

    if args.decompile {
        println!("{}", parse::decompile(&program));
    } else {
        let mut machine = Machine::new(program);

        debug!("Running program");

        loop {
            match machine.run() {
                RunState::Continue => {
                    continue;
                }

                RunState::BufferedOutput(s) => {
                    print!("{s}");
                }

                RunState::InuptNeeded => {
                    let mut line = String::new();
                    let stdin = io::stdin();
                    stdin.lock().read_line(&mut line).unwrap();

                    machine.push_input(line);
                }

                RunState::Halt => {
                    debug!("program execution complete");
                    break;
                }

                RunState::Error(e) => {
                    error!("{e}");
                    break;
                }
            }
        }
    }
}
