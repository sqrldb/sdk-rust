//! Wire protocol types and serialization for SquirrelDB.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Protocol magic bytes
pub const MAGIC: &[u8; 4] = b"SQRL";

/// Current protocol version
pub const PROTOCOL_VERSION: u8 = 0x01;

/// Maximum message size (16MB)
pub const MAX_MESSAGE_SIZE: u32 = 16 * 1024 * 1024;

/// Handshake status codes
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HandshakeStatus {
  Success = 0x00,
  VersionMismatch = 0x01,
  AuthFailed = 0x02,
}

impl TryFrom<u8> for HandshakeStatus {
  type Error = ();
  fn try_from(v: u8) -> Result<Self, Self::Error> {
    match v {
      0x00 => Ok(Self::Success),
      0x01 => Ok(Self::VersionMismatch),
      0x02 => Ok(Self::AuthFailed),
      _ => Err(()),
    }
  }
}

/// Message types
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MessageType {
  Request = 0x01,
  Response = 0x02,
  Notification = 0x03,
}

impl TryFrom<u8> for MessageType {
  type Error = ();
  fn try_from(v: u8) -> Result<Self, Self::Error> {
    match v {
      0x01 => Ok(Self::Request),
      0x02 => Ok(Self::Response),
      0x03 => Ok(Self::Notification),
      _ => Err(()),
    }
  }
}

/// Encoding formats
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Encoding {
  #[default]
  MessagePack = 0x01,
  Json = 0x02,
}

impl TryFrom<u8> for Encoding {
  type Error = ();
  fn try_from(v: u8) -> Result<Self, Self::Error> {
    match v {
      0x01 => Ok(Self::MessagePack),
      0x02 => Ok(Self::Json),
      _ => Err(()),
    }
  }
}

/// Protocol flags in handshake
#[derive(Debug, Clone, Copy, Default)]
pub struct ProtocolFlags {
  pub messagepack: bool,
  pub json_fallback: bool,
}

impl From<u8> for ProtocolFlags {
  fn from(byte: u8) -> Self {
    Self {
      messagepack: byte & 0x01 != 0,
      json_fallback: byte & 0x02 != 0,
    }
  }
}

impl From<ProtocolFlags> for u8 {
  fn from(flags: ProtocolFlags) -> u8 {
    let mut byte = 0u8;
    if flags.messagepack {
      byte |= 0x01;
    }
    if flags.json_fallback {
      byte |= 0x02;
    }
    byte
  }
}

/// Client-to-server message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ClientMessage {
  Query {
    id: String,
    query: String,
  },
  Subscribe {
    id: String,
    query: String,
  },
  Unsubscribe {
    id: String,
  },
  Insert {
    id: String,
    collection: String,
    data: serde_json::Value,
  },
  Update {
    id: String,
    collection: String,
    document_id: Uuid,
    data: serde_json::Value,
  },
  Delete {
    id: String,
    collection: String,
    document_id: Uuid,
  },
  ListCollections {
    id: String,
  },
  Ping {
    id: String,
  },
}

/// Server-to-client message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ServerMessage {
  Result { id: String, data: serde_json::Value },
  Change { id: String, change: ChangeEvent },
  Subscribed { id: String },
  Unsubscribed { id: String },
  Error { id: String, error: String },
  Pong { id: String },
}

/// Change event types for subscriptions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ChangeEvent {
  Initial { document: Document },
  Insert { new: Document },
  Update { old: serde_json::Value, new: Document },
  Delete { old: Document },
}

/// Document structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
  pub id: Uuid,
  pub collection: String,
  pub data: serde_json::Value,
  pub created_at: String,
  pub updated_at: String,
}
