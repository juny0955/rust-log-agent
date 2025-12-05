# log-agent
Log file detect agent with rust  
Detecting multiple log files to send central server

## Features
- Detecting Multiple Log Files
- TOML-based Configuration
- Strategy pattern for transmission 
  - Now Support: HTTP
  - Planned: WebSocket or else..

## Configuration
config file name: `log-agent.config`  

### Global Config 
```toml
[global]
end_point = "http://localhost:8080/log"
send_type = "HTTP"
retry = 3
retry_delay_ms = 100
```

| Key              | Type   | Description                              | Default | Required |
| ---------------- | ------ | ---------------------------------------- |---------|----------|
| `end_point`      | string | Server endpoint to send log data to      | –       | ✅        |
| `send_type`      | string | Transmission type (`HTTP`, more planned) | -       | ✅        |
| `retry`          | int    | Retry count on request error             | `3`     | ❌        |
| `retry_delay_ms` | int    | Delay (ms) between retries               | `100`   | ❌        |

### Source Config 
```toml
[[sources]]
name = "app1"
log_path = "app1.log"
delay_ms = 100 

[[sources]]
name = "app2"
log_path = "app2.log"
```

| Key        | Type   | Description                              | Default | Required |
| ---------- | ------ | ---------------------------------------- | ------- | -------- |
| `name`     | string | Logical name of this log source (unique) | –       | ✅        |
| `log_path` | string | Path to the log file to watch            | –       | ✅        |
| `delay_ms` | int    | Polling interval (ms) for file watching  | `500`   | ❌        |

### Example
```toml
[global]
end_point = "http://localhost:8080/log"
send_type = "HTTP"
retry = 5
retry_delay_ms = 200

[[sources]]
name = "app1"
log_path = "app1.log"
delay_ms = 100 

[[sources]]
name = "app2"
log_path = "app2.log"
```
