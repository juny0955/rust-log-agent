use crate::config::config::load_config;
use crate::sender::log_data::LogData;
use crate::sender::log_sender::build_sender;
use crate::watcher::watcher::Watcher;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread::JoinHandle;
use std::{process, thread};
use tracing::{error, info};

mod watcher;
mod sender;
mod config;

fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .init();

    info!("log-agent started");

    let sources = match load_config() {
        Ok(sources) => sources,
        Err(e) => {
            error!("{e}");
            process::exit(1);
        }
    };

    let (tx, rx) = mpsc::sync_channel::<LogData>(500);

    let sender_worker_handle = start_sender_worker(rx);

    let mut watcher_worker_handles: Vec<JoinHandle<()>> = Vec::new();
    for source in sources {
        let tx_clone = tx.clone();
        let watcher = Watcher::build(source, tx_clone)
            .unwrap_or_else(|err| {
                error!("Failed to build watcher: {err}");
                panic!("cannot build watcher");
            });

        watcher_worker_handles.push(start_watcher_worker(watcher));
    }

    drop(tx);

    for handle in watcher_worker_handles {
        handle.join().unwrap();
    }

    sender_worker_handle.join().unwrap();
}

fn start_sender_worker(rx: Receiver<LogData>) -> JoinHandle<()> {
    let sender = build_sender();

    thread::spawn(move || {
        for log_data in rx {
            sender.send(log_data);
        }
    })
}

fn start_watcher_worker(mut watcher: Watcher) -> JoinHandle<()>{
    thread::spawn(move || {
        if let Err(e) = watcher.watch() {
            error!("Watch error: {e}");
        }
    })
}
