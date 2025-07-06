use log::error;
use simple_mqtt::{MessageType, MqttMessage};
use std::io::{self, Write};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use uuid::Uuid;

struct Publisher {
    client_id: String,
}

impl Publisher {
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

    async fn publish(
        &self,
        stream: &mut TcpStream,
        topic: &str,
        payload: &str,
    ) -> anyhow::Result<()> {
        let message = MqttMessage::new(
            MessageType::Publish,
            topic.to_string(),
            Some(payload.to_string()),
            self.client_id.clone(),
        );

        let serialized = message.serialize()?;
        stream.write_all(serialized.as_bytes()).await?;
        stream.write_all(b"\n").await?;

        println!("Published to topic '{}': {}", topic, payload);
        Ok(())
    }

    async fn run(&self) -> anyhow::Result<()> {
        println!("MQTT Publisher");
        println!("Commands:");
        println!("  publish <broker_ip> <topic> <message>");
        println!("  exit");
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

            if input == "exit" {
                println!("Exiting publisher...");
                break;
            }

            let parts: Vec<&str> = input.split_whitespace().collect();

            if parts.len() < 4 {
                println!("Usage: publish <broker_ip> <topic> <message>");
                continue;
            }

            if parts[0] != "publish" {
                println!("Unknown command. Use 'publish <broker_ip> <topic> <message>' or 'exit'");
                continue;
            }

            let broker_addr = format!("{}:1883", parts[1]);
            let topic = parts[2];
            let message = parts[3..].join(" ");

            match self.connect(&broker_addr).await {
                Ok(mut stream) => {
                    if let Err(e) = self.publish(&mut stream, topic, &message).await {
                        error!("Failed to publish message: {}", e);
                        println!("Failed to publish message: {}", e);
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

    let publisher = Publisher::new();
    publisher.run().await?;

    Ok(())
}
