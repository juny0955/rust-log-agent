use crate::config::config::{global_config, load_config};
use crate::config::source_config::SourceConfig;
use crate::detector::detect_error::DetectError;
use crate::detector::detector::Detector;
use crate::sender::log_data::LogData;
use crate::sender::log_sender::build_sender;
use std::process::ExitCode;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, SyncSender};
use std::thread::JoinHandle;
use std::{io, thread};
use tracing::{error, info};

mod detector;
mod sender;
mod config;

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_target(false)
        .init();

    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => {
            println!("Press Enter to exit..");
            let _ = io::stdin().read_line(&mut String::new());
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<(), ExitCode> {
    info!("log-agent started");

    let sources = load_config()
        .map_err(|e| {
            error!("{e}");
            ExitCode::from(1)
        })?;

    let (tx, rx) = mpsc::sync_channel::<LogData>(global_config().channel_bound);
    let sender_worker_handle = start_sender_worker(rx);
    let detector_worker_handles = start_watcher_worker(tx, sources)
        .map_err(|e| {
            error!("Failed to build detector: {e}");
            ExitCode::from(1)
        })?;

    for handle in detector_worker_handles {
        handle.join().unwrap();
    }

    sender_worker_handle.join().unwrap();

    Ok(())
}

fn start_sender_worker(rx: Receiver<LogData>) -> JoinHandle<()> {
    let sender = build_sender();

    thread::spawn(move || {
        for log_data in rx {
            sender.send(log_data);
        }
    })
}

fn start_watcher_worker(tx: SyncSender<LogData>, sources: Vec<SourceConfig>) -> Result<Vec<JoinHandle<()>>, DetectError> {
    let mut handles: Vec<JoinHandle<()>> = Vec::new();

    for source in sources {
        let tx_clone = tx.clone();
        let mut detector = Detector::build(source, tx_clone)?;

        let handle = thread::spawn(move || {
            if let Err(e) = detector.detect() {
                error!("detect error: {e}");
            }
        });

        handles.push(handle);
    }

    Ok(handles)
}
