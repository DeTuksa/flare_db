use std::{
    fs::{File, OpenOptions},
    io::{self, BufRead, BufReader, Write},
    path::Path,
};

pub struct AppendOnlyLog {
    log_file: File,
}

impl AppendOnlyLog {
    pub fn new(log_path: &Path) -> io::Result<Self> {
        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(log_path)?;

        Ok(AppendOnlyLog { log_file })
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
                Err(e) => eprintln!("Failed to read log line: {:?}", e)
            }
        }
        Ok(commands)
    }
}
