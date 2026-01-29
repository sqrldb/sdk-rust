//! SquirrelDB Rust Client SDK
//!
//! A native TCP client for SquirrelDB, a realtime document database.
//!
//! # Example
//!
//! ```no_run
//! use squirreldb::{SquirrelDB, query::{table, field, SortDir}};
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
//!         "email": "alice@example.com",
//!         "age": 25
//!     })).await?;
//!
//!     println!("Inserted: {:?}", doc);
//!
//!     // Query with native builder (find/sort/limit)
//!     let query = table("users")
//!         .find(field("age").gt(21.0))
//!         .sort("name", SortDir::Asc)
//!         .limit(10)
//!         .compile();
//!
//!     let users: Vec<serde_json::Value> = client.query(&query).await?;
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

pub mod cache;
mod client;
mod error;
pub mod protocol;
pub mod query;
pub mod storage;

pub use client::{ConnectOptions, SquirrelDB, Subscription};
pub use error::{Error, Result};
pub use protocol::{
  ChangeEvent, ClientMessage, Document, Encoding, HandshakeStatus, MessageType, ProtocolFlags,
  ServerMessage, MAGIC, MAX_MESSAGE_SIZE, PROTOCOL_VERSION,
};
pub use query::{and, field, not, or, table, Field, Filter, QueryBuilder, SortDir, SortSpec};
pub use storage::{
  Bucket, MultipartUpload, Object, StorageClient, StorageError, StorageOptions, UploadPart,
};
pub use cache::{CacheClient, CacheError, CacheOptions, RespValue};
