//! Basic example demonstrating SquirrelDB Rust SDK usage.

use serde_json::json;
use squirreldb::SquirrelDB;

#[tokio::main]
async fn main() -> squirreldb::Result<()> {
  // Connect to SquirrelDB server
  let client = SquirrelDB::connect("localhost:8082").await?;
  println!("Connected! Session ID: {}", client.session_id());

  // Ping the server
  client.ping().await?;
  println!("Ping successful!");

  // List collections
  let collections = client.list_collections().await?;
  println!("Collections: {:?}", collections);

  // Insert a document
  let doc = client
    .insert(
      "users",
      json!({
          "name": "Alice",
          "email": "alice@example.com",
          "active": true
      }),
    )
    .await?;
  println!("Inserted document: {:?}", doc);

  // Query documents
  let users: serde_json::Value = client
    .query_raw(r#"db.table("users").filter(u => u.active).run()"#)
    .await?;
  println!("Active users: {}", serde_json::to_string_pretty(&users)?);

  // Update the document
  let updated = client
    .update(
      "users",
      doc.id,
      json!({
          "name": "Alice Updated",
          "email": "alice.updated@example.com",
          "active": true
      }),
    )
    .await?;
  println!("Updated document: {:?}", updated);

  // Subscribe to changes (in a real app, you'd run this in a separate task)
  println!("\nSubscribing to user changes...");
  println!("(Insert/update/delete users from another client to see changes)");
  println!("Press Ctrl+C to exit.\n");

  let mut sub = client.subscribe(r#"db.table("users").changes()"#).await?;

  while let Some(change) = sub.next().await {
    match change {
      squirreldb::ChangeEvent::Initial { document } => {
        println!("Initial: {}", document.data);
      }
      squirreldb::ChangeEvent::Insert { new } => {
        println!("Insert: {}", new.data);
      }
      squirreldb::ChangeEvent::Update { old, new } => {
        println!("Update: {} -> {}", old, new.data);
      }
      squirreldb::ChangeEvent::Delete { old } => {
        println!("Delete: {}", old.data);
      }
    }
  }

  Ok(())
}
