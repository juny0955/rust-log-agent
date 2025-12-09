use crate::config::source_config::SourceConfig;
use crate::sender::log_data::LogData;
use crate::detector::detect_error::DetectError;
use crate::detector::detect_event::DetectEvent;
use std::fs::{metadata, File};
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::sync::mpsc::SyncSender;
use std::thread;
use std::time::Duration;
use tracing::{error, info, trace, warn};

pub struct Detector {
    source: SourceConfig,
    current_len: u64,
    reader: BufReader<File>,
    tx: SyncSender<LogData>,
}

impl Detector {
    pub fn build(source: SourceConfig, tx: SyncSender<LogData>) -> Result<Self, DetectError> {
        let (reader, current_len) = Self::open_reader_at_end(&source.log_path)?;

        Ok(Self { source, current_len, reader, tx })
    }

    pub fn detect(&mut self) -> Result<(), DetectError> {
        info!("name: {} started detecting ", self.source.name);
        let watching_delay = Duration::from_millis(self.source.delay);

        loop {
            match self.next_event() {
                Ok(DetectEvent::NewLine(line)) => self.handle_newline(&line)?,
                Ok(DetectEvent::Rotated) => self.handle_rotate()?,
                Ok(DetectEvent::EndOfFile) => thread::sleep(watching_delay),
                Err(e) => match e {
                    DetectError::Recoverable(e) => {
                        warn!("{e}");
                        thread::sleep(watching_delay);
                        continue;
                    },
                    DetectError::UnRecoverable(_) | DetectError::ChannelClosed => {
                        return Err(e);
                    },
                }
            }
        }
    }

    fn handle_newline(&self, line: &str) -> Result<(), DetectError> {
        if line.is_empty() { return Ok(()); }

        trace!("detected new line on name: {}", &self.source.name);
        if let Err(e) = self.tx.send(LogData::new(&self.source.name, &line)) {
            error!("send err please restart process. {e}");
            return Err(DetectError::ChannelClosed);
        }

        Ok(())
    }

    fn handle_rotate(&mut self) -> Result<(), DetectError> {
        let (reader, current_len) = Self::open_reader_at_end(&self.source.log_path)?;

        self.reader = reader;
        self.current_len = current_len;

        Ok(())
    }

    fn open_reader_at_end(path: &str) -> Result<(BufReader<File>, u64), DetectError> {
        let file = File::open(path).map_err(DetectError::from_io_error)?;
        let meta = file.metadata().map_err(DetectError::from_io_error)?;
        let current_len = meta.len();

        let mut reader = BufReader::new(file);
        reader.seek(SeekFrom::End(0)).map_err(DetectError::from_io_error)?;

        Ok((reader, current_len))
    }

    fn next_event(&mut self) -> Result<DetectEvent, DetectError> {
        let mut buf = String::new();
        let bytes = self.reader.read_line(&mut buf).map_err(DetectError::from_io_error)?;

        if bytes == 0 {
            let new_meta = metadata(&self.source.log_path).map_err(DetectError::from_io_error)?;
            let new_meta_len = new_meta.len();

            if new_meta_len < self.current_len { return Ok(DetectEvent::Rotated); }

            return Ok(DetectEvent::EndOfFile);
        }

        self.current_len += bytes as u64;
        let trim_len = buf.trim_end_matches(['\n', '\r']).len();
        buf.truncate(trim_len);

        Ok(DetectEvent::NewLine(buf))
    }
}

#[cfg(test)]
mod tests {}
