use crate::config::source_config::SourceConfig;
use crate::sender::log_sender::LogSender;
use crate::watcher::watch_error::WatchError;
use crate::watcher::watch_event::WatchEvent;
use std::fs::{metadata, File};
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tracing::{info, warn};

const WATCH_DELAY_MS: u64 = 500;

pub struct Watcher {
    source: SourceConfig,
    current_len: u64,
    reader: BufReader<File>,
    sender: Arc<dyn LogSender>,
}

impl Watcher {
    pub fn build(source: SourceConfig, sender: Arc<dyn LogSender>) -> Result<Self, WatchError> {
        let (reader, current_len) = Self::open_reader_at_end(&source.log_path)?;

        Ok(Self { source, current_len, reader, sender })
    }

    pub fn watch(&mut self) -> Result<(), WatchError> {
        info!("name: {} is watching started", self.source.name);

        loop {
            match self.next_event() {
                Ok(WatchEvent::NewLine(line)) => {
                    if line.is_empty() { continue; }

                    self.sender.send(&self.source.name, &line);
                },
                Ok(WatchEvent::Rotated) => { self.handle_rotate()?; },
                Ok(WatchEvent::EndOfFile) => { thread::sleep(Duration::from_millis(WATCH_DELAY_MS)); }
                Err(e) => match e {
                    WatchError::Recoverable(e) => {
                        warn!("{e}");
                        thread::sleep(Duration::from_millis(WATCH_DELAY_MS));
                        continue;
                    },
                    WatchError::UnRecoverable(e) => {
                        return Err(WatchError::UnRecoverable(e));
                    },
                }
            }
        }
    }

    fn open_reader_at_end(path: &str) -> Result<(BufReader<File>, u64), WatchError> {
        let file = File::open(path).map_err(WatchError::from_io_error)?;
        let meta = file.metadata().map_err(WatchError::from_io_error)?;
        let current_len = meta.len();

        let mut reader = BufReader::new(file);
        reader.seek(SeekFrom::End(0)).map_err(WatchError::from_io_error)?;

        Ok((reader, current_len))
    }

    fn handle_rotate(&mut self) -> Result<(), WatchError> {
        let (reader, current_len) = Self::open_reader_at_end(&self.source.log_path)?;

        self.reader = reader;
        self.current_len = current_len;

        Ok(())
    }

    fn next_event(&mut self) -> Result<WatchEvent, WatchError> {
        let mut buf = String::new();
        let bytes = self.reader.read_line(&mut buf).map_err(WatchError::from_io_error)?;

        if bytes == 0 {
            let new_meta = metadata(&self.source.log_path).map_err(WatchError::from_io_error)?;
            let new_meta_len = new_meta.len();

            if new_meta_len < self.current_len { return Ok(WatchEvent::Rotated); }

            return Ok(WatchEvent::EndOfFile);
        }

        self.current_len += bytes as u64;
        let trim_len = buf.trim_end_matches(['\n', '\r']).len();
        buf.truncate(trim_len);

        Ok(WatchEvent::NewLine(buf))
    }
}

#[cfg(test)]
mod tests {}
