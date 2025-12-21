use crate::{
    config::{global_config, load_config},
    log_event::LogEvent,
    sender::payload::Payload,
};
use tokio::sync::mpsc;
use tracing::error;

mod config;
mod detector;
mod event_bucket;
mod log_event;
mod sender;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_target(false).init();

    // load configuration
    let sources = match load_config() {
        Ok(sources) => sources,
        Err(e) => {
            error!("{e}");
            return;
        }
    };

    // create mpsc
    let channel_bound = global_config().channel_bound;

    // detector -> aggregator
    let (event_sender, event_receiver) = mpsc::channel::<LogEvent>(channel_bound);
    // aggregator -> sender
    let (payload_sender, payload_receiver) = mpsc::channel::<Payload>(channel_bound);

    let detector_handles = match detector::spawn_detectors(event_sender, sources) {
        Ok(hs) => hs,
        Err(e) => {
            error!("{e}");
            return;
        }
    };

    let aggregator_handle = event_bucket::spawn_event_aggregator(event_receiver, payload_sender);

    let sender_handle = match sender::spawn_sender(payload_receiver).await {
        Ok(h) => h,
        Err(e) => {
            error!("{e}");
            return;
        }
    };

    for detector_handle in detector_handles {
        let _ = detector_handle.join();
    }

    let _ = aggregator_handle.await;
    let _ = sender_handle.await;

    error!("Detectors All Closed Process Exit..");
}
