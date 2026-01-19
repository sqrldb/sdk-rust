//! SquirrelDB Rust Client SDK
//!
//! A native TCP client for SquirrelDB, a realtime document database.
//!
//! # Example
//!
//! ```no_run
//! use squirreldb::SquirrelDB;
//! use serde_json::json;
//!
//! #[tokio::main]
//! async fn main() -> squirreldb::Result<()> {
//!     // Connect to SquirrelDB
//!     let client = SquirrelDB::connect("localhost:8082").await?;
//!
//!     // Insert a document
//!     let doc = client.insert("users", json!({
//!         "name": "Alice",
//!         "email": "alice@example.com"
//!     })).await?;
//!
//!     println!("Inserted: {:?}", doc);
//!
//!     // Query documents
//!     let users: Vec<serde_json::Value> = client.query(
//!         r#"db.table("users").filter(u => u.name === "Alice").run()"#
//!     ).await?;
//!
//!     println!("Found: {:?}", users);
//!
//!     // Subscribe to changes
//!     let mut sub = client.subscribe(r#"db.table("users").changes()"#).await?;
//!     while let Some(change) = sub.next().await {
//!         println!("Change: {:?}", change);
//!     }
//!
//!     Ok(())
//! }
//! ```

mod client;
mod error;
pub mod protocol;

pub use client::{ConnectOptions, SquirrelDB, Subscription};
pub use error::{Error, Result};
pub use protocol::{
  ChangeEvent, ClientMessage, Document, Encoding, HandshakeStatus, MessageType, ProtocolFlags,
  ServerMessage, MAGIC, MAX_MESSAGE_SIZE, PROTOCOL_VERSION,
};
