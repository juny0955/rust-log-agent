# Log-Agent
Log file detect agent with rust  
Detecting multiple log files to send central server

## Features
- Detecting Multiple Log Files
- TOML-based Configuration
- Strategy pattern for transmission 
  - Now Support: HTTP
  - Planned: WebSocket, MQ, Kafka or else..

## Configuration
config file name: `log-agent.config`

### Global Config 
```toml
[global]
agent_name = "agent1"
end_point = "http://localhost:8080/log"
send_type = "HTTP"
max_send_task = 10
retry_count = 3
retry_delay_ms = 100
channel_bound = 2048
```

| Key              | Type   | Description                                 | Default | Required |
|------------------|--------|---------------------------------------------|---------|----------|
| `agent_name`     | String | Setting Agent Name                          | -       | ✅        |
| `end_point`      | String | Server endpoint to send log data to         | –       | ✅        |
| `send_type`      | String | Transmission type (`HTTP`, more planned)    | -       | ✅        |
| `max_send_task`  | String | Setting sending task size                   | `5`     | ❌        |
| `retry_count`    | u32    | Retry count on request error                | `3`     | ❌        |
| `retry_delay_ms` | u64    | Delay (ms) between retries                  | `100`   | ❌        |
| `channel_bound`  | usize  | Buffer size for the detector's sync channel | `1024`  | ❌        |

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
|------------|--------|------------------------------------------|---------|----------|
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
