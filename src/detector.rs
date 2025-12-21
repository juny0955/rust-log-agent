use crate::{config::SourceConfig, log_event::LogEvent};
use std::{
    fs::{metadata, File},
    io::{BufRead, BufReader, Seek, SeekFrom},
    mem, thread,
    time::Duration,
};
use tokio::sync::mpsc::Sender;
use tracing::{error, info, trace, warn};

pub mod detect_error;
pub use detect_error::DetectError;

mod detect_event;
use detect_event::DetectEvent;

pub struct Detector {
    source: SourceConfig,
    current_len: u64,
    reader: BufReader<File>,
    buf: String,
    event_sender: Sender<LogEvent>,
}

impl Detector {
    pub fn build(
        source: SourceConfig,
        event_sender: Sender<LogEvent>,
    ) -> Result<Self, DetectError> {
        let (reader, current_len) = Self::open_reader_at_end(&source.log_path)?;

        Ok(Self {
            source,
            current_len,
            reader,
            event_sender,
            buf: String::with_capacity(1024),
        })
    }

    pub fn detect(&mut self) -> Result<(), DetectError> {
        info!("[{}] Started detecting ", self.source.name);
        let watching_delay = Duration::from_millis(self.source.delay_ms);

        loop {
            match self.next_event() {
                Ok(DetectEvent::NewLine(line)) => self.handle_newline(line)?,
                Ok(DetectEvent::Rotated) => self.handle_rotate()?,
                Ok(DetectEvent::EndOfFile) => thread::sleep(watching_delay),
                Err(e) => match e {
                    DetectError::Recoverable(e) => {
                        warn!("[{}] {e}", self.source.name);
                        thread::sleep(watching_delay);
                        continue;
                    }
                    DetectError::UnRecoverable(_) | DetectError::ChannelClosed(_) => {
                        return Err(e);
                    }
                },
            }
        }
    }

    fn handle_newline(&self, log: String) -> Result<(), DetectError> {
        if log.is_empty() {
            return Ok(());
        }
        trace!("[{}] detected new line", &self.source.name);

        self.event_sender
            .blocking_send(LogEvent::new(self.source.name.clone(), log))?;

        Ok(())
    }

    fn handle_rotate(&mut self) -> Result<(), DetectError> {
        info!("[{}] is rotated", self.source.name);
        let (reader, current_len) = Self::open_reader_at_end(&self.source.log_path)?;

        self.reader = reader;
        self.current_len = current_len;

        Ok(())
    }

    fn open_reader_at_end(path: &str) -> Result<(BufReader<File>, u64), DetectError> {
        let file = File::open(path)?;
        let meta = file.metadata()?;
        let current_len = meta.len();

        let mut reader = BufReader::new(file);
        reader.seek(SeekFrom::End(0))?;

        Ok((reader, current_len))
    }

    fn next_event(&mut self) -> Result<DetectEvent, DetectError> {
        self.buf.clear();
        let bytes = self.reader.read_line(&mut self.buf)?;

        if bytes == 0 {
            let new_len = metadata(&self.source.log_path)?.len();
            if new_len < self.current_len {
                return Ok(DetectEvent::Rotated);
            }

            self.current_len = new_len;
            return Ok(DetectEvent::EndOfFile);
        }

        while matches!(self.buf.as_bytes().last(), Some(b'\n' | b'\r')) {
            self.buf.pop();
        }

        let line = mem::replace(&mut self.buf, String::with_capacity(1024));
        Ok(DetectEvent::NewLine(line))
    }
}

pub fn spawn_detectors(event_sender: Sender<LogEvent>, sources: Vec<SourceConfig>) -> Result<Vec<thread::JoinHandle<()>>, DetectError> {
    let mut detector_handles: Vec<thread::JoinHandle<()>> = Vec::new();

    for source in sources {
        let mut detector = Detector::build(source, event_sender.clone())?;

        let detector_handle = thread::spawn(move || {
            if let Err(e) = detector.detect() {
                error!("Detecting error: {e}");
            }
        });

        detector_handles.push(detector_handle);
    }

    Ok(detector_handles)
}

#[cfg(test)]
mod tests {}
