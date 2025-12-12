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
use tokio_tungstenite::tungstenite::Utf8Bytes;
use tokio_tungstenite::{
    connect_async,
    tungstenite::Message,
    MaybeTlsStream,
    WebSocketStream
};
use tracing::{debug, error, info, warn};

mod ws_error;

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
type WsWriter = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;

pub struct WebSocketSenderStrategy {
    ws_writer: WsWriter,
}

impl WebSocketSenderStrategy {
    pub async fn build() -> Result<Self, String> {
        let global_config = global_config();
        let endpoint = &global_config.end_point;
        let max_retry = global_config.retry;
        let retry_delay = Duration::from_millis(global_config.retry_delay_ms);

        for attempt in 1..=max_retry {
            match Self::try_connect(endpoint).await {
                Ok(ws_stream) => {
                    info!("Success to connect server");
                    let (ws_writer, _ws_read) = ws_stream.split();

                    return Ok(WebSocketSenderStrategy { ws_writer });
                },
                Err(WsError::Retryable(e)) => {
                    if attempt == max_retry {
                        error!("Failed to connect after retry {max_retry} msg: {e}");
                        return Err(e);
                    }

                    warn!("Failed to connect msg: {e}, retry...{attempt}/{max_retry} ");
                    time::sleep(retry_delay).await;
                },
                Err(WsError::NonRetryable(e)) => {
                    error!("Failed to connect(non-retry): {e}");
                    return Err(e);
                },
            };
        }

        Err("retry is end".to_string())
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

        let global_config = global_config();
        let max_retry = global_config.retry;
        let retry_delay = Duration::from_millis(global_config.retry_delay_ms);

        for attempt in 1..=max_retry {
            match self.try_send(text.as_str()).await {
                Ok(()) => {
                    debug!("[{}] send success. on attempt: {attempt}/{max_retry}", log_data.name);
                    return;
                }
                Err(WsError::NonRetryable(e)) => {
                    error!("Failed to send(non-retry): {e}");
                    return;
                }
                Err(WsError::Retryable(e)) => {
                    if attempt == max_retry {
                        // TODO try reconnect
                        error!("[{}] Failed to send after {max_retry} msg: {e}", log_data.name);
                        return;
                    }

                    warn!("[{}] Failed to send : {e}, retry...{attempt}/{max_retry}", log_data.name);
                    time::sleep(retry_delay).await;
                }
            };
        }
    }
}