use std::{
    fs::{self, File},
    io::{self, BufRead, BufReader}, collections::VecDeque,
};

use clap::Parser;
use log::{debug, error};
use parse::parse_16_bit_little_endian;

use machine::{Machine, RunState};
use replay::{ReplayManager, REPLAY_SAVE_DIR};

mod machine;
mod parse;
mod replay;

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
        return;
    }

    let mut replay_manager = ReplayManager::new();

    let mut autoplay_commands = VecDeque::new();

    if let Some(last_replay) = ReplayManager::replay_files().expect("Error reading replay files").last() {
        let replay_file = File::open(format!("{REPLAY_SAVE_DIR}/{last_replay}")).expect(&format!("Error opening replay file {last_replay}"));
        for line in BufReader::new(replay_file).lines() {
            autoplay_commands.push_back(line.expect(&format!("Error reading replay file {last_replay}")));        
        }
    }

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
                if let Some(command) = autoplay_commands.front() {
                    println!("Replay command:{command}");
                }

                let mut line = String::new();
                let stdin = io::stdin();
                stdin.lock().read_line(&mut line).unwrap();

                if line == "\n" {
                    if let Some(command) = autoplay_commands.pop_front() {
                        println!("{command}");
                        line = format!("{command}\n");
                    }
                }

                let line = replay_manager.push(line).expect("Error storing input line");

                dbg!(&line);

                machine.push_input(&line);
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

    replay_manager
        .save(&ReplayManager::next_file_path().expect("Error getting replay file path"))
        .unwrap();
}
