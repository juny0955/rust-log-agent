use crate::sender::ws_sender::ws_error::WsError;
use crate::{
    config::global_config,
    sender::{LogData, Sender}
};
use async_trait::async_trait;
use futures::{
    stream::SplitSink,
    SinkExt,
    StreamExt
};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time;
use tokio_tungstenite::{
    connect_async,
    tungstenite::Message,
    MaybeTlsStream,
    WebSocketStream
};
use tokio_tungstenite::tungstenite::Utf8Bytes;
use tracing::{error, warn};

mod ws_error;

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
type WsWriter = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;

pub struct WebSocketSenderStrategy {
    ws_writer: WsWriter,
}

impl WebSocketSenderStrategy {
    pub async fn new() -> Result<Self, WsError> {
        let global_config = global_config();
        let endpoint = &global_config.end_point;
        let max_retry = global_config.retry;
        let retry_delay = Duration::from_millis(global_config.retry_delay_ms);

        for attempt in 1..=max_retry {
            match Self::try_connect(endpoint).await {
                Ok(ws_stream) => {
                    let (ws_writer, _ws_read) = ws_stream.split();

                    return Ok(WebSocketSenderStrategy { ws_writer });
                },
                Err(WsError::Retryable(e)) => {
                    if attempt == max_retry {
                        error!("Failed to connect after {max_retry} msg: {e}");
                        return Err(WsError::NonRetryable(e));
                    }
                    warn!("Failed to connect after {max_retry} msg: {e}");

                    time::sleep(retry_delay).await;
                    continue;
                },
                Err(WsError::NonRetryable(e)) => {
                    error!("Failed to connect(non-retry): {e}");
                    return Err(WsError::NonRetryable(e));
                },
            };
        }
    }

    async fn try_connect(endpoint: &str) -> Result<WsStream, WsError> {
        let (ws_stream, _response) = connect_async(endpoint).await?;

        Ok(ws_stream)
    }

    async fn try_send(&mut self, text: &str) -> Result<(), WsError> {
        self.ws_writer.send(Message::Text(Utf8Bytes::from(text))).await?;

        Ok(())
    }
}

#[async_trait]
impl Sender for WebSocketSenderStrategy {
    async fn send(&mut self, log_data: LogData) {
        let text = serde_json::to_string(&log_data).unwrap();

        if let Err(e) =self.try_send(text.as_str()).await {
            match e {
                WsError::Retryable(e) => {}
                WsError::NonRetryable(e) => {
                    error!("Failed to connect(non-retry): {e}");
                    return;
                }
            }
        };
    }
}