use crate::config::config::load_config;
use crate::sender::log_sender::build_sender;
use crate::watcher::watcher::Watcher;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;

mod watcher;
mod sender;
mod config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sources = load_config()?;
    let sender = build_sender();

    let mut handles: Vec<JoinHandle<()>> = Vec::new();

    for source in sources {
        let sender = Arc::clone(&sender);

        let handle = thread::spawn(move || {
            let mut watcher = match Watcher::build(source, sender) {
                Ok(w) => w,
                Err(e) => {
                    eprintln!("failed to build watcher: {e}");
                    return;
                }
            };

            if let Err(e) = watcher.watch() {
                eprintln!("watch error: {e}")
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        if let Err(e) = handle.join() {
            eprintln!("thread join error: {e:?}");
        }
    }

    Ok(())
}

