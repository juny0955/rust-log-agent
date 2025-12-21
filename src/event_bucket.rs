use std::{collections::HashMap, time::Duration};

use crate::{
    config::global_config,
    log_event::LogEvent,
    sender::payload::{Logs, Payload, Source},
};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task;

pub struct EventBucket {
    bucket: HashMap<String, Vec<Logs>>,
    max_batch_size: u8,
    total_size: u8,
}

impl EventBucket {
    pub fn new() -> Self {
        Self {
            bucket: HashMap::new(),
            max_batch_size: global_config().max_batch_size,
            total_size: 0,
        }
    }

    // insert bucket and check size if full of max_batch_size
    pub fn receive(&mut self, event: LogEvent) -> Option<()> {
        let source_name = event.name.clone();
        let logs = Logs::from_event(event);

        self.bucket.entry(source_name).or_default().push(logs);

        self.total_size += 1;

        if self.total_size >= self.max_batch_size {
            return Some(());
        }

        None
    }

    pub fn is_empty(&self) -> bool {
        self.bucket.is_empty()
    }

    pub fn drain_to_payload(&mut self) -> Payload {
        self.total_size = 0;

        let log_datas = self
            .bucket
            .drain()
            .map(|(name, logs)| Source::new(name, logs))
            .collect();

        Payload::new(log_datas)
    }
}

pub fn spawn_event_aggregator(
    mut event_receiver: Receiver<LogEvent>,
    payload_sender: Sender<Payload>,
) -> task::JoinHandle<()> {
    let mut event_bucket = EventBucket::new();

    let interval = Duration::from_secs(global_config().interval_secs);
    let mut ticker = tokio::time::interval(interval);

    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    if !event_bucket.is_empty() {
                        let payload = event_bucket.drain_to_payload();
                        if payload_sender.send(payload).await.is_err() {
                            break;
                        }
                    }
                }
                receive = event_receiver.recv() => {
                    match receive {
                        Some(event) => {
                            if event_bucket.receive(event).is_some() {
                                let payload = event_bucket.drain_to_payload();
                                if payload_sender.send(payload).await.is_err() {
                                    break;
                                }
                            }
                        }
                        None => {
                            if !event_bucket.is_empty() {
                                let payload = event_bucket.drain_to_payload();
                                if payload_sender.send(payload).await.is_err() {
                                    break;
                                }
                            }

                            break;
                        }
                    }
                }
            }
        }
    })
}
