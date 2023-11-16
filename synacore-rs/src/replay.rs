use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use regex::Regex;

pub const REPLAY_SAVE_DIR: &str = "replays";

pub struct ReplayManager {
    commands: Vec<String>,
}

impl ReplayManager {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn push(&mut self, command: String) -> Option<&String> {
        self.commands.push(command);

        self.commands.last()
    }

    pub fn save(self, file_path: &Path) -> std::io::Result<()> {
        let replay_dir_path = Path::new(REPLAY_SAVE_DIR);

        if replay_dir_path.try_exists()? == false {
            std::fs::create_dir_all(replay_dir_path)?;
        }

        let mut file = File::create(file_path)?;

        for command in self.commands {
            file.write_all(command.as_bytes())?;
        }

        Ok(())
    }

    pub fn replay_files() -> std::io::Result<Vec<String>> {
        let replay_dir_path = Path::new(REPLAY_SAVE_DIR);

        if replay_dir_path.try_exists()? == false {
            return Ok(vec![]);
        }

        let replay_file_regex = Regex::new(r"(replay_[\d]+)").unwrap();

        let mut replay_files = replay_dir_path
            .read_dir()?
            .filter_map(|e| {
                if let Ok(e) = e {
                    if e.file_type().ok()?.is_file() {
                        let path = e.file_name().into_string().ok()?;
                        if replay_file_regex.is_match(&path) {
                            return Some(path);
                        }
                    }
                }

                None
            })
            .collect::<Vec<String>>();

        replay_files.sort();

        Ok(replay_files)
        
    }

    pub fn next_file_path() -> std::io::Result<PathBuf> {
        let mut replay_files = Self::replay_files()?;

        // return 1 higher then the highest numbered replay file found
        while let Some(replay_file) = replay_files.pop() {
            if let Some((_, file_num)) = replay_file.split_once('_') {
                if let Ok(file_num) = file_num.parse::<u32>() {
                    return Ok(PathBuf::from(&format!(
                        "{REPLAY_SAVE_DIR}/replay_{}",
                        file_num + 1
                    )));
                }
            }
        }

        // If no replay files were found, return default file path
        return Ok(PathBuf::from(&format!("{REPLAY_SAVE_DIR}/replay_1")));
    }
}
