use crate::config::config::load_config;
use crate::sender::log_sender::build_sender;
use crate::watcher::watcher::Watcher;
use std::sync::Arc;
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

    let sender = build_sender();

    let mut handles: Vec<JoinHandle<()>> = Vec::new();
    for source in sources {
        let sender = Arc::clone(&sender);

        let handle = thread::spawn(move || {
            let mut watcher = match Watcher::build(source, sender) {
                Ok(w) => w,
                Err(e) => {
                    error!("Failed to build watcher: {e}");
                    return;
                }
            };

            if let Err(e) = watcher.watch() {
                error!("Watch error: {e}");
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
