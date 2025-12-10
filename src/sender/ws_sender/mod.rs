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
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async,
    tungstenite::Message,
    MaybeTlsStream,
    WebSocketStream
};

type WsWriter = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;

pub struct WebSocketSender {
    ws_writer: WsWriter,
}

impl WebSocketSender {
    pub async fn new() -> Self {
        let global_config = global_config();
        let (ws_stream, _response) = connect_async(&global_config.end_point).await
            .expect("Failed to Connect"); // TODO Retry

        let (ws_writer, _ws_read) = ws_stream.split();

        WebSocketSender { ws_writer }
    }
}

#[async_trait]
impl Sender for WebSocketSender {
    async fn send(&mut self, log_data: LogData) {
        let text = serde_json::to_string(&log_data).unwrap();

        self.ws_writer.send(Message::Text(text.into())).await
            .expect("Failed to send"); // TODO Retry
    }
}