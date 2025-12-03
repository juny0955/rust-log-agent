use crate::config::source_config::SourceConfig;
use crate::sender::log_sender::LogSender;
use std::fs::{metadata, File};
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::sync::Arc;
use std::time::Duration;
use std::{io, thread};
use crate::watcher::watch_event::WatchEvent;

pub struct Watcher {
    source: SourceConfig,
    current_len: u64,
    reader: BufReader<File>,
    sender: Arc<dyn LogSender>,
}

impl Watcher {
    pub fn build(source: SourceConfig, sender: Arc<dyn LogSender>) -> io::Result<Self> {
        let (reader, current_len) = Self::open_reader_at_end(&source.log_path)?;

        Ok(Self {
            source,
            current_len,
            reader,
            sender,
        })
    }

    pub fn watch(&mut self) -> io::Result<()> {
        loop {
            match self.next_event()? {
                WatchEvent::NewLine(line) => {
                    if line.is_empty() {
                        continue;
                    }

                    self.sender.send(&self.source.name, &line);
                },
                WatchEvent::Rotated => {
                    self.handle_rotate()?;
                },
                WatchEvent::EndOfFile => {
                    thread::sleep(Duration::from_millis(500));
                }
            }
        }
    }

    fn open_reader_at_end(path: &str) -> io::Result<(BufReader<File>, u64)> {
        let file = File::open(path)?;
        let meta = file.metadata()?;
        let current_len = meta.len();

        let mut reader = BufReader::new(file);
        reader.seek(SeekFrom::End(0))?;

        Ok((reader, current_len))
    }

    fn handle_rotate(&mut self) -> io::Result<()> {
        let (reader, current_len) = Self::open_reader_at_end(&self.source.log_path)?;

        self.reader = reader;
        self.current_len = current_len;

        Ok(())
    }

    fn next_event(&mut self) -> io::Result<WatchEvent> {
        let mut buf = String::new();
        let bytes = self.reader.read_line(&mut buf)?;

        if bytes == 0 {
            let new_meta = metadata(&self.source.log_path)?;
            let new_meta_len = new_meta.len();

            if new_meta_len < self.current_len {
                return Ok(WatchEvent::Rotated);
            }

            self.current_len = new_meta_len;
            return Ok(WatchEvent::EndOfFile);
        }

        let trim_len = buf.trim_end_matches(['\n', '\r']).len();
        buf.truncate(trim_len);

        Ok(WatchEvent::NewLine(buf))
    }
}
