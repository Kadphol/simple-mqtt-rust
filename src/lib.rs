use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Subscribe,
    Publish,
    Unsubscribe,
    Disconnect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttMessage {
    pub msg_type: MessageType,
    pub topic: String,
    pub payload: Option<String>,
    pub client_id: String,
}

#[derive(Debug, Clone)]
pub struct Client {
    pub id: String,
    pub addr: SocketAddr,
    pub sender: mpsc::UnboundedSender<String>,
}

pub type Topics = HashMap<String, Vec<Client>>;

impl MqttMessage {
    pub fn new(
        msg_type: MessageType,
        topic: String,
        payload: Option<String>,
        client_id: String,
    ) -> Self {
        Self {
            msg_type,
            topic,
            payload,
            client_id,
        }
    }

    pub fn serialize(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn deserialize(data: &str) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(data)?)
    }
}

impl Client {
    pub fn new(addr: SocketAddr, sender: mpsc::UnboundedSender<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            addr,
            sender,
        }
    }
}
