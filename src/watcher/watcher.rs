use crate::config::source_config::SourceConfig;
use crate::sender::log_data::LogData;
use crate::watcher::watch_error::WatchError;
use crate::watcher::watch_event::WatchEvent;
use std::fs::{metadata, File};
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::sync::mpsc::SyncSender;
use std::thread;
use std::time::Duration;
use tracing::{error, info, warn};

pub struct Watcher {
    source: SourceConfig,
    current_len: u64,
    reader: BufReader<File>,
    tx: SyncSender<LogData>,
}

impl Watcher {
    pub fn build(source: SourceConfig, tx: SyncSender<LogData>) -> Result<Self, WatchError> {
        let (reader, current_len) = Self::open_reader_at_end(&source.log_path)?;

        Ok(Self { source, current_len, reader, tx })
    }

    pub fn watch(&mut self) -> Result<(), WatchError> {
        info!("name: {} is watching started", self.source.name);
        let watching_delay = Duration::from_millis(self.source.delay);

        loop {
            match self.next_event() {
                Ok(WatchEvent::NewLine(line)) => self.handle_newline(&line)?,
                Ok(WatchEvent::Rotated) => self.handle_rotate()?,
                Ok(WatchEvent::EndOfFile) => thread::sleep(watching_delay),
                Err(e) => match e {
                    WatchError::Recoverable(e) => {
                        warn!("{e}");
                        thread::sleep(watching_delay);
                        continue;
                    },
                    WatchError::UnRecoverable(_) | WatchError::ChannelClosed => {
                        return Err(e);
                    },
                }
            }
        }
    }

    fn handle_newline(&self, line: &str) -> Result<(), WatchError>{
        if line.is_empty() { return Ok(()); }

        if let Err(e) = self.tx.send(LogData::new(&self.source.name, &line)) {
            error!("send err please restart process. {e}");
            return Err(WatchError::ChannelClosed);
        }

        Ok(())
    }

    fn handle_rotate(&mut self) -> Result<(), WatchError> {
        let (reader, current_len) = Self::open_reader_at_end(&self.source.log_path)?;

        self.reader = reader;
        self.current_len = current_len;

        Ok(())
    }

    fn open_reader_at_end(path: &str) -> Result<(BufReader<File>, u64), WatchError> {
        let file = File::open(path).map_err(WatchError::from_io_error)?;
        let meta = file.metadata().map_err(WatchError::from_io_error)?;
        let current_len = meta.len();

        let mut reader = BufReader::new(file);
        reader.seek(SeekFrom::End(0)).map_err(WatchError::from_io_error)?;

        Ok((reader, current_len))
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
