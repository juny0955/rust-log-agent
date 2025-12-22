# Log-Agent
A lightweight, high-performance log file monitoring agent written in Rust.  
Detects changes in multiple log files and sends them to a central server with batching and retry capabilities.

## Features
- **Multi-file Detection** - Monitor multiple log files simultaneously
- **Log Rotation Support** - Automatically detects and handles log file rotation
- **Batching** - Aggregates logs before sending to reduce network overhead
- **Exponential Backoff Retry** - Intelligent retry mechanism with exponential backoff
- ⚙**TOML Configuration** - Simple and flexible configuration
- **Async/Concurrent** - Built on Tokio for high performance
- **Strategy Pattern** - Pluggable transmission strategies
  -  HTTP/HTTPS (currently supported)
  -  Planned: WebSocket, MQ, Kafka

## Architecture

```
┌─────────────┐     ┌───────────────┐     ┌──────────┐     ┌────────┐
│  Detector   │────▶│   Aggregator  │────▶│  Sender  │────▶│ Server │
│  (Thread)   │     │ (Batch+Timer) │     │  (Task)  │     │        │
└─────────────┘     └───────────────┘     └──────────┘     └────────┘
 Multiple files         LogEvent             Payload            │
                     (mpsc channel)        (Semaphore)          │
                                                │               │
                                                │               │ On Failure
                                                ▼               │
                                         ┌─────────────┐        │
                                         │ Retry Queue │◀───────┘
                                         │(Worker Pool)│
                                         └─────────────┘
                                               │
                                               └──────▶ Exponential Backoff Retry
```

### Components

1. **Detector** - OS thread per log file, detects new lines and rotation
2. **Aggregator** - Batches logs and sends based on size or time interval
3. **Sender** - Handles concurrent transmission with semaphore control
4. **Retry Worker Pool** - Processes failed requests with exponential backoff

## How It Works

1. **Detection**: Each configured log file is monitored by a dedicated thread
   - Reads new lines as they appear
   - Detects log rotation (file size decrease)
   - Sends `LogEvent` to the aggregator via mpsc channel

2. **Aggregation**: Collects logs from multiple sources
   - Batches up to `max_batch_size` logs
   - Sends batch every `interval_secs` seconds (whichever comes first)
   - Creates a `Payload` with agent name and grouped logs

3. **Transmission**: Sends batched logs to the server
   - Concurrent requests limited by `max_send_task` (via Semaphore)
   - First attempt: immediate send
   - Failed retryable requests: queued for retry

4. **Retry with Exponential Backoff**:
   - Worker pool processes retry queue (`max_send_task` workers)
   - Delay calculation: `base_delay * 2^(attempt-1)` (capped at 30s)
   - Example: 100ms → 200ms → 400ms → 800ms → 1600ms

## Installation

### From Source
```bash
git clone https://github.com/your-repo/log-agent
cd log-agent
cargo build --release
```

The binary will be located at `target/release/log-agent`.

## Usage

1. Create a configuration file named `log-agent.config`:

```toml
[global]
agent_name = "my-server"
end_point = "http://localhost:8080/log"
send_type = "HTTP"

[[sources]]
name = "app1"
log_path = "/var/log/app1.log"

[[sources]]
name = "app2"
log_path = "/var/log/app2.log"
```

2. Run the agent:

```bash
./log-agent
```

The agent will start monitoring the specified log files and send logs to the configured endpoint.

## Configuration

Config file name: `log-agent.config`

### Global Config

```toml
[global]
agent_name = "agent1"
end_point = "http://localhost:8080/log"
send_type = "HTTP"
max_send_task = 5
retry_count = 3
retry_delay_ms = 100
channel_bound = 1024
interval_secs = 5
max_batch_size = 100
```

| Key              | Type   | Description                                                      | Default | Required |
|------------------|--------|------------------------------------------------------------------|---------|----------|
| `agent_name`     | String | Unique identifier for this agent                                 | -       | ✅        |
| `end_point`      | String | Server endpoint to send log data to                              | -       | ✅        |
| `send_type`      | String | Transmission type (`HTTP`, more planned)                         | -       | ✅        |
| `max_send_task`  | u8     | Max concurrent send tasks (controls parallelism)                 | `5`     | ❌        |
| `retry_count`    | u8     | Maximum retry attempts on failure                                | `3`     | ❌        |
| `retry_delay_ms` | u64    | Base delay (ms) for exponential backoff retry                    | `100`   | ❌        |
| `channel_bound`  | usize  | Buffer size for internal mpsc channels                           | `1024`  | ❌        |
| `interval_secs`  | u64    | Time interval (seconds) to send batched logs                     | `5`     | ❌        |
| `max_batch_size` | u8     | Maximum number of logs per batch (triggers immediate send)       | `100`   | ❌        |

#### Retry with Exponential Backoff

When a transmission fails with a retryable error (network issues, 5xx errors), the agent uses **exponential backoff**:

- **Formula**: `delay = retry_delay_ms * 2^(attempt-1)`
- **Max delay**: Capped at 30 seconds
- **Example** (retry_delay_ms = 100, retry_count = 5):
  - Attempt 1: 100ms
  - Attempt 2: 200ms
  - Attempt 3: 400ms
  - Attempt 4: 800ms
  - Attempt 5: 1600ms

This prevents overwhelming the server during outages while ensuring eventual delivery.

#### Concurrency Control

- `max_send_task` controls both:
  1. Initial send concurrency (via Semaphore)
  2. Retry worker pool size

- **Total max connections** = ~2x `max_send_task` (during high failure rates)
- **Recommended values**:
  - Small: 2-5 (default: 5)
  - Medium: 5-10
  - Large: 10-20

### Source Config

```toml
[[sources]]
name = "app1"
log_path = "app1.log"
delay_ms = 100
```

| Key        | Type   | Description                              | Default | Required |
|------------|--------|------------------------------------------|---------|----------|
| `name`     | string | Logical name of this log source (unique) | -       | ✅        |
| `log_path` | string | Path to the log file to watch            | -       | ✅        |
| `delay_ms` | u64    | Polling interval (ms) for file watching  | `500`   | ❌        |

### Complete Example

```toml
[global]
agent_name = "production-server-1"
end_point = "https://log-collector.example.com/api/logs"
send_type = "HTTP"
max_send_task = 10
retry_count = 5
retry_delay_ms = 200
channel_bound = 2048
interval_secs = 3
max_batch_size = 50

[[sources]]
name = "nginx-access"
log_path = "/var/log/nginx/access.log"
delay_ms = 100

[[sources]]
name = "nginx-error"
log_path = "/var/log/nginx/error.log"
delay_ms = 100

[[sources]]
name = "app"
log_path = "/var/log/myapp/app.log"
delay_ms = 200
```

## Payload Format

Logs are sent as JSON in the following format:

```json
{
  "agentName": "agent1",
  "sources": [
    {
      "sourceName": "app1",
      "logs": [
        {
          "data": "2024-01-15 10:30:00 INFO Starting application",
          "timestamp": "2024-01-15T10:30:00.123Z"
        },
        {
          "data": "2024-01-15 10:30:01 INFO Application started",
          "timestamp": "2024-01-15T10:30:01.456Z"
        }
      ]
    },
    {
      "sourceName": "app2",
      "logs": [
        {
          "data": "2024-01-15 10:30:02 ERROR Connection failed",
          "timestamp": "2024-01-15T10:30:02.789Z"
        }
      ]
    }
  ]
}
```

- **agentName**: Identifier from global config
- **sources**: Array of log sources
- **sourceName**: Name from source config
- **logs**: Array of log entries
- **data**: Raw log line content
- **timestamp**: UTC timestamp when the log was detected (RFC 3339 format)

## Log Rotation Handling

The agent automatically detects log rotation by monitoring file size:
- If the file size decreases, rotation is detected
- The agent reopens the file and continues from the beginning
- No logs are lost during rotation

## Dependencies

- **tokio** - Async runtime
- **reqwest** - HTTP client
- **serde** - Serialization
- **chrono** - Timestamp handling
- **tracing** - Logging and diagnostics
- **toml** - Configuration parsing

