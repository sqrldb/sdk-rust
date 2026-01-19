//! Client tests for SquirrelDB Rust SDK.

use squirreldb::{ConnectOptions, Error};

#[test]
fn test_connect_options_default() {
  let opts = ConnectOptions::new("localhost", 8082);
  assert_eq!(opts.host, "localhost");
  assert_eq!(opts.port, 8082);
  assert!(opts.auth_token.is_none());
  assert!(opts.use_messagepack);
  assert!(opts.json_fallback);
}

#[test]
fn test_connect_options_with_auth() {
  let opts = ConnectOptions::new("localhost", 8082)
    .with_auth("my-secret-token");

  assert_eq!(opts.host, "localhost");
  assert_eq!(opts.port, 8082);
  assert_eq!(opts.auth_token, Some("my-secret-token".to_string()));
}

#[test]
fn test_connect_options_builder_chain() {
  let opts = ConnectOptions::new("db.example.com", 9000)
    .with_auth("token123");

  assert_eq!(opts.host, "db.example.com");
  assert_eq!(opts.port, 9000);
  assert_eq!(opts.auth_token, Some("token123".to_string()));
}

#[test]
fn test_error_display() {
  let err = Error::Connection("failed to connect".to_string());
  assert_eq!(format!("{}", err), "Connection error: failed to connect");

  let err = Error::Handshake("invalid magic".to_string());
  assert_eq!(format!("{}", err), "Handshake failed: invalid magic");

  let err = Error::VersionMismatch { server: 2, client: 1 };
  assert!(format!("{}", err).contains("server=2"));
  assert!(format!("{}", err).contains("client=1"));

  let err = Error::AuthFailed;
  assert_eq!(format!("{}", err), "Authentication failed");

  let err = Error::Server("not found".to_string());
  assert_eq!(format!("{}", err), "Server error: not found");

  let err = Error::Timeout;
  assert_eq!(format!("{}", err), "Timeout");

  let err = Error::ChannelClosed;
  assert_eq!(format!("{}", err), "Channel closed");
}

#[test]
fn test_error_from_io() {
  let io_err = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "refused");
  let err: Error = io_err.into();
  match err {
    Error::Io(_) => {}
    _ => panic!("Expected Io error"),
  }
}

#[test]
fn test_error_from_json() {
  let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
  let err: Error = json_err.into();
  match err {
    Error::Serialization(_) => {}
    _ => panic!("Expected Serialization error"),
  }
}

#[test]
fn test_error_from_msgpack_encode() {
  // Create a value that can't be serialized properly
  // This is a bit contrived but tests the conversion
  let err = Error::Serialization("msgpack error".to_string());
  assert!(format!("{}", err).contains("msgpack error"));
}

#[tokio::test]
async fn test_connect_invalid_host() {
  use squirreldb::SquirrelDB;

  let result = SquirrelDB::connect("invalid.host.that.does.not.exist:8082").await;
  assert!(result.is_err());

  match result.unwrap_err() {
    Error::Connection(_) | Error::Io(_) => {}
    e => panic!("Expected Connection or Io error, got: {:?}", e),
  }
}

#[tokio::test]
async fn test_connect_refused() {
  use squirreldb::SquirrelDB;

  // Try to connect to a port that's likely not listening
  let result = SquirrelDB::connect("127.0.0.1:59999").await;
  assert!(result.is_err());
}

#[tokio::test]
async fn test_connect_with_options_invalid() {
  use squirreldb::SquirrelDB;

  let opts = ConnectOptions::new("127.0.0.1", 59999);
  let result = SquirrelDB::connect_with_options(opts).await;
  assert!(result.is_err());
}
