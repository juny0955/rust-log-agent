# log-agent
Log file detect agent with rust  
Detecting multiple log files to send central server
---
## Features
- Detecting Multiple Log Files
- TOML-based Configuration
- Strategy pattern for transmission 
  - Now Support: HTTP
  - Planned: WebSocket or else..

## Configuration
config file name: `log-agent.config` 

example: 
```toml
[global]
end_point = "http://localhost:8080/log"
send_type = "HTTP"

[[sources]]
name = "app1"
log_path = "app1.log"

[[sources]]
name = "app2"
log_path = "app2.log"
```