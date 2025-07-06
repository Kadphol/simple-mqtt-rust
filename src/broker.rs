mod lib;
use log::{error, info, warn};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};

#[derive(Clone)]
pub struct Broker {
    topics: Arc<Mutex<lib::Topics>>,
    clients: Arc<Mutex<HashMap<String, lib::Client>>>,
}

impl Broker {
    pub fn new() -> Self {
        Self {
            topics: Arc::new(Mutex::new(HashMap::new())),
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn start(&self, addr: &str) -> anyhow::Result<()> {
        let listener = TcpListener::bind(addr).await?;
        info!("MQTT Broker started on {}", addr);
        println!("MQTT Broker started on {}", addr);
        println!("Waiting for connections...");

        loop {
            let (stream, client_addr) = listener.accept().await?;
            info!("New client connected: {}", client_addr);
            println!("New client connected: {}", client_addr);

            let broker = self.clone();
            tokio::spawn(async move {
                if let Err(e) = broker.handle_client(stream, client_addr).await {
                    error!("Error handling client {}: {}", client_addr, e);
                }
            });
        }
    }

    async fn handle_client(&self, mut stream: TcpStream, addr: SocketAddr) -> anyhow::Result<()> {
        let (rx, mut tx) = stream.split();
        let mut reader = BufReader::new(rx);
        let (sender, mut receiver) = mpsc::unbounded_channel();

        let client = lib::Client::new(addr, sender);
        let client_id = client.id.clone();

        // Add client to the broker
        {
            let mut clients = self.clients.lock().await;
            clients.insert(client_id.clone(), client);
        }

        // Spawn task to handle outgoing messages
        let mut tx_clone = tx.clone();
        tokio::spawn(async move {
            while let Some(message) = receiver.recv().await {
                if let Err(e) = tx_clone.write_all(message.as_bytes()).await {
                    error!("Failed to send message: {}", e);
                    break;
                }
                if let Err(e) = tx_clone.write_all(b"\n").await {
                    error!("Failed to send newline: {}", e);
                    break;
                }
            }
        });

        // Handle incoming messages
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    info!("Client {} disconnected", addr);
                    break;
                }
                Ok(_) => {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }

                    match lib::MqttMessage::deserialize(line) {
                        Ok(msg) => {
                            if let Err(e) = self.process_message(msg, &client_id).await {
                                error!("Error processing message: {}", e);
                            }
                        }
                        Err(e) => {
                            warn!("Invalid message format from {}: {}", addr, e);
                        }
                    }
                }
                Err(e) => {
                    error!("Error reading from client {}: {}", addr, e);
                    break;
                }
            }
        }

        // Cleanup when client disconnects
        self.cleanup_client(&client_id).await;
        Ok(())
    }

    async fn process_message(&self, msg: lib::MqttMessage, client_id: &str) -> anyhow::Result<()> {
        match msg.msg_type {
            lib::MessageType::Subscribe => {
                info!("Client {} subscribing to topic: {}", client_id, msg.topic);
                println!("Client {} subscribing to topic: {}", client_id, msg.topic);
                self.subscribe_client(client_id, &msg.topic).await?;
            }
            lib::MessageType::Publish => {
                info!("Publishing to topic {}: {:?}", msg.topic, msg.payload);
                println!("Publishing to topic {}: {:?}", msg.topic, msg.payload);
                self.publish_message(&msg.topic, msg.payload.as_deref().unwrap_or(""))
                    .await?;
            }
            lib::MessageType::Unsubscribe => {
                info!(
                    "Client {} unsubscribing from topic: {}",
                    client_id, msg.topic
                );
                println!(
                    "Client {} unsubscribing from topic: {}",
                    client_id, msg.topic
                );
                self.unsubscribe_client(client_id, &msg.topic).await?;
            }
            lib::MessageType::Disconnect => {
                info!("Client {} requested disconnect", client_id);
                println!("Client {} requested disconnect", client_id);
                self.cleanup_client(client_id).await;
            }
        }
        Ok(())
    }

    async fn subscribe_client(&self, client_id: &str, topic: &str) -> anyhow::Result<()> {
        let mut topics = self.topics.lock().await;
        let clients = self.clients.lock().await;

        if let Some(client) = clients.get(client_id) {
            let subscribers = topics.entry(topic.to_string()).or_insert_with(Vec::new);

            // Check if client is already subscribed
            if !subscribers.iter().any(|c| c.id == client_id) {
                subscribers.push(client.clone());
                info!("Client {} subscribed to topic: {}", client_id, topic);
            }
        }
        Ok(())
    }

    async fn publish_message(&self, topic: &str, payload: &str) -> anyhow::Result<()> {
        let topics = self.topics.lock().await;

        if let Some(subscribers) = topics.get(topic) {
            let message = format!("Topic: {} | Message: {}", topic, payload);

            for client in subscribers {
                if let Err(e) = client.sender.send(message.clone()) {
                    warn!("Failed to send message to client {}: {}", client.id, e);
                }
            }

            info!(
                "Message published to {} subscribers on topic: {}",
                subscribers.len(),
                topic
            );
        } else {
            info!("No subscribers for topic: {}", topic);
        }
        Ok(())
    }

    async fn unsubscribe_client(&self, client_id: &str, topic: &str) -> anyhow::Result<()> {
        let mut topics = self.topics.lock().await;

        if let Some(subscribers) = topics.get_mut(topic) {
            subscribers.retain(|client| client.id != client_id);

            if subscribers.is_empty() {
                topics.remove(topic);
            }
        }
        Ok(())
    }

    async fn cleanup_client(&self, client_id: &str) {
        // Remove client from all topics
        let mut topics = self.topics.lock().await;
        for (_, subscribers) in topics.iter_mut() {
            subscribers.retain(|client| client.id != client_id);
        }

        // Remove empty topics
        topics.retain(|_, subscribers| !subscribers.is_empty());

        // Remove client from clients list
        let mut clients = self.clients.lock().await;
        clients.remove(client_id);

        info!("Client {} cleaned up", client_id);
        println!("Client {} disconnected and cleaned up", client_id);
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let broker = Broker::new();
    broker.start("127.0.0.1:1883").await?;

    Ok(())
}
