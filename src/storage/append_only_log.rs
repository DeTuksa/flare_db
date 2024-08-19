use std::{
    collections::HashMap,
    fs::{remove_file, rename, File, OpenOptions},
    io::{self, BufRead, BufReader, Result, Write},
    path::Path,
};

pub struct AppendOnlyLog {
    log_file: File,
    log_path: String,
}

impl AppendOnlyLog {
    pub fn new(log_path: &Path) -> io::Result<Self> {
        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(log_path)?;

        Ok(AppendOnlyLog {
            log_file,
            log_path: log_path.to_str().unwrap().to_string(),
        })
    }

    pub fn append(&mut self, command: &str) -> io::Result<()> {
        writeln!(self.log_file, "{}", command)?;
        self.log_file.flush()
    }

    pub fn replay(&self) -> io::Result<Vec<String>> {
        let reader = BufReader::new(&self.log_file);
        let mut commands = Vec::new();

        for line in reader.lines() {
            match line {
                Ok(command) => commands.push(command),
                Err(e) => eprintln!("Failed to read log line: {:?}", e),
            }
        }
        Ok(commands)
    }

    pub fn compact(&mut self, current_state: &HashMap<String, String>) -> Result<()> {
        let temp_log_path = format!("{}.tmp", self.log_path);
        let mut temp_log_file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(&temp_log_path)?;

        for (key, value) in current_state {
            writeln!(temp_log_file, "SET {} {}", key, value)?;
        }

        temp_log_file.flush()?;
        drop(temp_log_file);
        remove_file(&self.log_path)?;
        rename(temp_log_path, &self.log_path)?;

        self.log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(&self.log_path)?;
        Ok(())
    }
}
