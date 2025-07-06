# Simple MQTT in Rust

A simple MQTT (Message Queuing Telemetry Transport) implementation written in Rust, featuring a broker, publisher, and subscriber. This is a Rust rewrite of the original Python project.

## Features

- **Broker**: Central message broker that handles topic subscriptions and message routing
- **Publisher**: Client that publishes messages to specific topics
- **Subscriber**: Client that subscribes to topics and receives messages
- **Async/Await**: Built with Tokio for high-performance async I/O
- **JSON Messages**: Uses JSON for message serialization
- **Multiple Clients**: Supports multiple concurrent publishers and subscribers

## Project Structure

```
simple-mqtt/
├── Cargo.toml
├── src/
│   ├── lib.rs          # Shared library with common types
│   └── bin/
│       ├── broker.rs   # MQTT broker implementation
│       ├── publisher.rs # MQTT publisher client
│       └── subscriber.rs # MQTT subscriber client
└── README.md
```

## Dependencies

- `tokio` - Async runtime
- `serde` - Serialization framework
- `serde_json` - JSON serialization
- `clap` - Command-line argument parsing
- `uuid` - UUID generation for client IDs
- `log` - Logging framework
- `env_logger` - Environment-based logger
- `anyhow` - Error handling

## Building

```bash
cargo build --release
```

## Usage

### 1. Start the Broker

```bash
cargo run --bin broker
```

The broker will start on `127.0.0.1:1883` and display connection information.

### 2. Start a Subscriber

```bash
cargo run --bin subscriber
```

Then use the subscribe command:

```
> subscribe 127.0.0.1 /room1/light
```

### 3. Start a Publisher

```bash
cargo run --bin publisher
```

Then use the publish command:

```
> publish 127.0.0.1 /room1/light value=on
```

## Command Reference

### Broker

- Automatically starts on `127.0.0.1:1883`
- Displays connection and message information
- Press `Ctrl+C` to shutdown

### Publisher

- `publish <broker_ip> <topic> <message>` - Publish a message to a topic
- `exit` - Exit the publisher

### Subscriber

- `subscribe <broker_ip> <topic>` - Subscribe to a topic
- `Ctrl+C` - Exit the subscriber

## Example Session

1. **Terminal 1 (Broker):**

   ```bash
   cargo run --bin broker
   # Output: MQTT Broker started on 127.0.0.1:1883
   ```

2. **Terminal 2 (Subscriber):**

   ```bash
   cargo run --bin subscriber
   > subscribe 127.0.0.1 /room1/light
   # Output: Subscribed to topic: /room1/light
   #         Listening for messages... (Press Ctrl+C to exit)
   ```

3. **Terminal 3 (Publisher):**

   ```bash
   cargo run --bin publisher
   > publish 127.0.0.1 /room1/light value=on
   # Output: Published to topic '/room1/light': value=on
   ```

4. **Back to Terminal 2:**
   ```
   # Output: Received: Topic: /room1/light | Message: value=on
   ```

## Message Format

Messages are serialized as JSON with the following structure:

```json
{
  "msg_type": "Publish|Subscribe|Unsubscribe|Disconnect",
  "topic": "/topic/name",
  "payload": "message content",
  "client_id": "unique-client-id"
}
```

## Differences from Original Python Version

- **Async/Await**: Uses Tokio for asynchronous operations
- **Type Safety**: Rust's type system prevents many runtime errors
- **Memory Safety**: No garbage collection, but memory-safe
- **Performance**: Generally faster execution and lower memory usage
- **Error Handling**: More robust error handling with `anyhow`
- **Concurrency**: Built-in support for concurrent connections

## Logging

Set the log level using the `RUST_LOG` environment variable:

```bash
RUST_LOG=info cargo run --bin broker
```

## License

This project is open source and available under the MIT License.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## Future Enhancements

- Add QoS (Quality of Service) levels
- Implement retained messages
- Add authentication and authorization
- Support for wildcards in topic subscriptions
- Add configuration file support
- Implement proper MQTT protocol compliance
