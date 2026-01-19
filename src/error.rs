//! Error types for the SquirrelDB client SDK.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
  #[error("Connection error: {0}")]
  Connection(String),

  #[error("Handshake failed: {0}")]
  Handshake(String),

  #[error("Protocol version mismatch: server={server}, client={client}")]
  VersionMismatch { server: u8, client: u8 },

  #[error("Authentication failed")]
  AuthFailed,

  #[error("IO error: {0}")]
  Io(#[from] std::io::Error),

  #[error("Serialization error: {0}")]
  Serialization(String),

  #[error("Server error: {0}")]
  Server(String),

  #[error("Timeout")]
  Timeout,

  #[error("Channel closed")]
  ChannelClosed,
}

impl From<rmp_serde::encode::Error> for Error {
  fn from(e: rmp_serde::encode::Error) -> Self {
    Self::Serialization(e.to_string())
  }
}

impl From<rmp_serde::decode::Error> for Error {
  fn from(e: rmp_serde::decode::Error) -> Self {
    Self::Serialization(e.to_string())
  }
}

impl From<serde_json::Error> for Error {
  fn from(e: serde_json::Error) -> Self {
    Self::Serialization(e.to_string())
  }
}

pub type Result<T> = std::result::Result<T, Error>;
