use reqwest::blocking::Client;
use serde::Serialize;
use crate::config::config::global_config;
use crate::sender::log_sender::LogSender;

#[derive(Serialize)]
struct LogBody<'a> {
    name: &'a str,
    data: &'a str,
}

pub struct HttpSenderStrategy {
    client: Client,
}

impl HttpSenderStrategy {
    pub fn new() -> Self {
        let client = Client::new();
        Self { client }
    }
}

impl LogSender for HttpSenderStrategy {
    fn send(&self, name: &str, data: &str) {
        let body = LogBody{ name, data };

        match self.client
            .post(&global_config().end_point)
            .json(&body)
            .send()
        {
            Ok(response) => println!("send success: {}", response.status()),
            Err(e) => eprintln!("send error {e}"),
        }
    }
}