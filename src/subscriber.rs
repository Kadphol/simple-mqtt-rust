mod lib;

use log::error;
use std::io::{self, Write};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::signal;
use uuid::Uuid;

struct Subscriber {
    client_id: String,
}

impl Subscriber {
    fn new() -> Self {
        Self {
            client_id: Uuid::new_v4().to_string(),
        }
    }

    async fn connect(&self, broker_addr: &str) -> anyhow::Result<TcpStream> {
        let stream = TcpStream::connect(broker_addr).await?;
        println!("Connected to broker at {}", broker_addr);
        Ok(stream)
    }

    async fn subscribe(&self, stream: &mut TcpStream, topic: &str) -> anyhow::Result<()> {
        let message = lib::MqttMessage::new(
            lib::MessageType::Subscribe,
            topic.to_string(),
            None,
            self.client_id.clone(),
        );

        let serialized = message.serialize()?;
        stream.write_all(serialized.as_bytes()).await?;
        stream.write_all(b"\n").await?;

        println!("Subscribed to topic: {}", topic);
        Ok(())
    }

    async fn listen_for_messages(&self, stream: TcpStream) -> anyhow::Result<()> {
        let (rx, _tx) = stream.split();
        let mut reader = BufReader::new(rx);
        let mut line = String::new();

        println!("Listening for messages... (Press Ctrl+C to exit)");

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    println!("Connection closed by broker");
                    break;
                }
                Ok(_) => {
                    let message = line.trim();
                    if !message.is_empty() {
                        println!("Received: {}", message);
                    }
                }
                Err(e) => {
                    error!("Error reading message: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    async fn run(&self) -> anyhow::Result<()> {
        println!("MQTT Subscriber");
        println!("Commands:");
        println!("  subscribe <broker_ip> <topic>");
        println!("  Press Ctrl+C to exit");
        println!();

        loop {
            print!("> ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            if input.is_empty() {
                continue;
            }

            let parts: Vec<&str> = input.split_whitespace().collect();

            if parts.len() < 3 {
                println!("Usage: subscribe <broker_ip> <topic>");
                continue;
            }

            if parts[0] != "subscribe" {
                println!("Unknown command. Use 'subscribe <broker_ip> <topic>'");
                continue;
            }

            let broker_addr = format!("{}:1883", parts[1]);
            let topic = parts[2];

            match self.connect(&broker_addr).await {
                Ok(mut stream) => {
                    if let Err(e) = self.subscribe(&mut stream, topic).await {
                        error!("Failed to subscribe: {}", e);
                        println!("Failed to subscribe: {}", e);
                        continue;
                    }

                    // Start listening for messages
                    let listen_task = tokio::spawn(async move {
                        if let Err(e) = self.listen_for_messages(stream).await {
                            error!("Error listening for messages: {}", e);
                        }
                    });

                    // Wait for Ctrl+C
                    match signal::ctrl_c().await {
                        Ok(_) => {
                            println!("\nReceived Ctrl+C, exiting...");
                            listen_task.abort();
                            break;
                        }
                        Err(e) => {
                            error!("Error waiting for Ctrl+C: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to connect to broker: {}", e);
                    println!("Failed to connect to broker: {}", e);
                }
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let subscriber = Subscriber::new();
    subscriber.run().await?;

    Ok(())
}
