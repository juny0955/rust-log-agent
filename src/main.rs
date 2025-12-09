use crate::config::config::{global_config, load_config};
use crate::config::source_config::SourceConfig;
use crate::detector::detect_error::DetectError;
use crate::detector::detector::Detector;
use crate::sender::log_data::LogData;
use crate::sender::log_sender::build_sender;
use std::process::ExitCode;
use std::{io};
use task::{spawn_blocking, JoinHandle};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task;
use tracing::{error, info};

mod detector;
mod sender;
mod config;

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_target(false)
        .init();

    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => {
            println!("Press Enter to exit..");
            let _ = io::stdin().read_line(&mut String::new());
            ExitCode::from(1)
        }
    }
}

async fn run() -> Result<(), ExitCode> {
    info!("log-agent started");

    let sources = load_config()
        .map_err(|e| {
            error!("{e}");
            ExitCode::from(1)
        })?;

    let (tx, rx) = channel::<LogData>(global_config().channel_bound);
    let sender_worker_handle = start_sender_worker(rx);
    start_detector_worker(tx, sources)
        .map_err(|e| {
            error!("Failed to build detector: {e}");
            ExitCode::from(1)
        })?;

    sender_worker_handle.await
        .map_err(|e| {
            error!("Sender worker failed: {e:?}");
            ExitCode::from(1)
        })?;

    Ok(())
}

fn start_sender_worker(mut rx: Receiver<LogData>) -> JoinHandle<()> {
    let sender = build_sender();

    tokio::spawn(async move {
        while let Some(log_data) = rx.recv().await {
            sender.send(log_data).await;
        }
    })
}

fn start_detector_worker(tx: Sender<LogData>, sources: Vec<SourceConfig>) -> Result<(), DetectError> {
    for source in sources {
        let tx_clone = tx.clone();
        let mut detector = Detector::build(source, tx_clone)?;

        let _ = spawn_blocking(move || {
            if let Err(e) = detector.detect() {
                error!("detect error: {e}");
            }
        });
    }

    Ok(())
}
