# SquirrelDB Rust SDK

Official Rust client for SquirrelDB.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
squirreldb = "0.1"
```

## Quick Start

```rust
use squirreldb::{Client, Config};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::connect(Config {
        host: "localhost".into(),
        port: 8080,
        token: std::env::var("SQUIRRELDB_TOKEN").ok(),
    }).await?;

    // Insert a document
    let user = client.table("users")
        .insert(json!({
            "name": "Alice",
            "email": "alice@example.com"
        }))
        .await?;
    println!("Created user: {}", user["id"]);

    // Query documents
    let users = client.table("users")
        .filter("u => u.status === 'active'")
        .run()
        .await?;
    println!("Found {} active users", users.len());

    // Subscribe to changes
    let mut subscription = client.table("messages")
        .changes()
        .await?;

    while let Some(change) = subscription.next().await {
        println!("Change: {:?}", change);
    }

    Ok(())
}
```

## Documentation

Visit [squirreldb.com/docs/sdks](https://squirreldb.com/docs/sdks) for full documentation.

## License

Apache License 2.0 - see [LICENSE](LICENSE) for details.
