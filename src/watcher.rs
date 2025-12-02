use crate::config::source_config::SourceConfig;
use crate::sender::log_sender::LogSender;
use std::fs::{metadata, File};
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::sync::Arc;
use std::time::Duration;
use std::{io, thread};

pub struct Watcher {
    source: SourceConfig,
    current_len: u64,
    reader: BufReader<File>,
    sender: Arc<dyn LogSender>,
}

impl Watcher {
    pub fn build(source: SourceConfig, sender: Arc<dyn LogSender>) -> io::Result<Self> {
        let file = File::open(&source.log_path)?;
        let meta = file.metadata()?;
        let current_len = meta.len();

        let mut reader = BufReader::new(file);
        reader.seek(SeekFrom::End(0))?;

        Ok(Self {
            source,
            current_len,
            reader,
            sender,
        })
    }

    fn new_file(&mut self) -> io::Result<()> {
        let file = File::open(&self.source.log_path)?;
        let meta = file.metadata()?;
        let current_len = meta.len();

        let mut reader = BufReader::new(file);
        reader.seek(SeekFrom::End(0))?;

        self.current_len = current_len;
        self.reader = reader;
        Ok(())
    }

    fn update_current_len(&mut self, new_len: u64) {
        self.current_len = new_len;
    }

    pub fn watch(&mut self) -> io::Result<()> {
        loop {
            let mut buf = String::new();
            let bytes = self.reader.read_line(&mut buf)?;

            if bytes == 0 {
                let new_meta = metadata(&self.source.log_path)?;
                let new_meta_len = new_meta.len();

                if new_meta_len < self.current_len {
                    self.new_file()?;
                    continue;
                } else {
                    self.update_current_len(new_meta_len);
                }

                thread::sleep(Duration::from_millis(500));
                continue;
            }

            // remove CRLF
            // WINDOWS: \r\n
            // UNIX: \n
            let line = buf.trim_end_matches(['\n', '\r']);
            if line.is_empty() {
                continue;
            }

            self.sender.send(&self.source.name, line);
        }
    }
}