//! Protocol encoding/decoding tests for SquirrelDB Rust SDK.

use squirreldb::protocol::*;
use squirreldb::{ClientMessage, ServerMessage, ChangeEvent, Document};

#[test]
fn test_protocol_constants() {
  assert_eq!(MAGIC, b"SQRL");
  assert_eq!(PROTOCOL_VERSION, 0x01);
  assert_eq!(MAX_MESSAGE_SIZE, 16 * 1024 * 1024);
}

#[test]
fn test_handshake_status_conversion() {
  assert_eq!(HandshakeStatus::try_from(0x00), Ok(HandshakeStatus::Success));
  assert_eq!(HandshakeStatus::try_from(0x01), Ok(HandshakeStatus::VersionMismatch));
  assert_eq!(HandshakeStatus::try_from(0x02), Ok(HandshakeStatus::AuthFailed));
  assert!(HandshakeStatus::try_from(0xFF).is_err());
}

#[test]
fn test_message_type_conversion() {
  assert_eq!(MessageType::try_from(0x01), Ok(MessageType::Request));
  assert_eq!(MessageType::try_from(0x02), Ok(MessageType::Response));
  assert_eq!(MessageType::try_from(0x03), Ok(MessageType::Notification));
  assert!(MessageType::try_from(0x00).is_err());
  assert!(MessageType::try_from(0xFF).is_err());
}

#[test]
fn test_encoding_conversion() {
  assert_eq!(Encoding::try_from(0x01), Ok(Encoding::MessagePack));
  assert_eq!(Encoding::try_from(0x02), Ok(Encoding::Json));
  assert!(Encoding::try_from(0x00).is_err());
  assert!(Encoding::try_from(0xFF).is_err());
}

#[test]
fn test_protocol_flags_to_byte() {
  let flags = ProtocolFlags {
    messagepack: false,
    json_fallback: false,
  };
  assert_eq!(u8::from(flags), 0x00);

  let flags = ProtocolFlags {
    messagepack: true,
    json_fallback: false,
  };
  assert_eq!(u8::from(flags), 0x01);

  let flags = ProtocolFlags {
    messagepack: false,
    json_fallback: true,
  };
  assert_eq!(u8::from(flags), 0x02);

  let flags = ProtocolFlags {
    messagepack: true,
    json_fallback: true,
  };
  assert_eq!(u8::from(flags), 0x03);
}

#[test]
fn test_protocol_flags_from_byte() {
  let flags = ProtocolFlags::from(0x00);
  assert!(!flags.messagepack);
  assert!(!flags.json_fallback);

  let flags = ProtocolFlags::from(0x01);
  assert!(flags.messagepack);
  assert!(!flags.json_fallback);

  let flags = ProtocolFlags::from(0x02);
  assert!(!flags.messagepack);
  assert!(flags.json_fallback);

  let flags = ProtocolFlags::from(0x03);
  assert!(flags.messagepack);
  assert!(flags.json_fallback);
}

#[test]
fn test_client_message_query_serialization() {
  let msg = ClientMessage::Query {
    id: "123".to_string(),
    query: "db.table(\"users\").run()".to_string(),
  };

  // JSON serialization
  let json = serde_json::to_string(&msg).unwrap();
  assert!(json.contains("\"type\":\"query\""));
  assert!(json.contains("\"id\":\"123\""));
  assert!(json.contains("\"query\":"));

  // Deserialize back
  let parsed: ClientMessage = serde_json::from_str(&json).unwrap();
  match parsed {
    ClientMessage::Query { id, query } => {
      assert_eq!(id, "123");
      assert_eq!(query, "db.table(\"users\").run()");
    }
    _ => panic!("Expected Query message"),
  }
}

#[test]
fn test_client_message_insert_serialization() {
  let msg = ClientMessage::Insert {
    id: "456".to_string(),
    collection: "users".to_string(),
    data: serde_json::json!({"name": "Alice"}),
  };

  let json = serde_json::to_string(&msg).unwrap();
  assert!(json.contains("\"type\":\"insert\""));
  assert!(json.contains("\"collection\":\"users\""));

  let parsed: ClientMessage = serde_json::from_str(&json).unwrap();
  match parsed {
    ClientMessage::Insert { id, collection, data } => {
      assert_eq!(id, "456");
      assert_eq!(collection, "users");
      assert_eq!(data["name"], "Alice");
    }
    _ => panic!("Expected Insert message"),
  }
}

#[test]
fn test_client_message_update_serialization() {
  let doc_id = uuid::Uuid::new_v4();
  let msg = ClientMessage::Update {
    id: "789".to_string(),
    collection: "users".to_string(),
    document_id: doc_id,
    data: serde_json::json!({"name": "Bob"}),
  };

  let json = serde_json::to_string(&msg).unwrap();
  assert!(json.contains("\"type\":\"update\""));
  assert!(json.contains("\"document_id\":"));

  let parsed: ClientMessage = serde_json::from_str(&json).unwrap();
  match parsed {
    ClientMessage::Update { id, collection, document_id, data } => {
      assert_eq!(id, "789");
      assert_eq!(collection, "users");
      assert_eq!(document_id, doc_id);
      assert_eq!(data["name"], "Bob");
    }
    _ => panic!("Expected Update message"),
  }
}

#[test]
fn test_client_message_delete_serialization() {
  let doc_id = uuid::Uuid::new_v4();
  let msg = ClientMessage::Delete {
    id: "101".to_string(),
    collection: "users".to_string(),
    document_id: doc_id,
  };

  let json = serde_json::to_string(&msg).unwrap();
  assert!(json.contains("\"type\":\"delete\""));

  let parsed: ClientMessage = serde_json::from_str(&json).unwrap();
  match parsed {
    ClientMessage::Delete { id, collection, document_id } => {
      assert_eq!(id, "101");
      assert_eq!(collection, "users");
      assert_eq!(document_id, doc_id);
    }
    _ => panic!("Expected Delete message"),
  }
}

#[test]
fn test_client_message_subscribe_serialization() {
  let msg = ClientMessage::Subscribe {
    id: "sub1".to_string(),
    query: "db.table(\"users\").changes()".to_string(),
  };

  let json = serde_json::to_string(&msg).unwrap();
  assert!(json.contains("\"type\":\"subscribe\""));

  let parsed: ClientMessage = serde_json::from_str(&json).unwrap();
  match parsed {
    ClientMessage::Subscribe { id, query } => {
      assert_eq!(id, "sub1");
      assert!(query.contains("changes"));
    }
    _ => panic!("Expected Subscribe message"),
  }
}

#[test]
fn test_client_message_unsubscribe_serialization() {
  let msg = ClientMessage::Unsubscribe {
    id: "sub1".to_string(),
  };

  let json = serde_json::to_string(&msg).unwrap();
  assert!(json.contains("\"type\":\"unsubscribe\""));
  assert!(json.contains("\"id\":\"sub1\""));
}

#[test]
fn test_client_message_ping_serialization() {
  let msg = ClientMessage::Ping {
    id: "ping1".to_string(),
  };

  let json = serde_json::to_string(&msg).unwrap();
  assert!(json.contains("\"type\":\"ping\""));
}

#[test]
fn test_client_message_list_collections_serialization() {
  let msg = ClientMessage::ListCollections {
    id: "list1".to_string(),
  };

  let json = serde_json::to_string(&msg).unwrap();
  assert!(json.contains("\"type\":\"listcollections\""));
}

#[test]
fn test_server_message_result_serialization() {
  let msg = ServerMessage::Result {
    id: "123".to_string(),
    data: serde_json::json!([{"id": "doc1", "name": "Alice"}]),
  };

  let json = serde_json::to_string(&msg).unwrap();
  assert!(json.contains("\"type\":\"result\""));
  assert!(json.contains("\"data\":"));

  let parsed: ServerMessage = serde_json::from_str(&json).unwrap();
  match parsed {
    ServerMessage::Result { id, data } => {
      assert_eq!(id, "123");
      assert!(data.is_array());
    }
    _ => panic!("Expected Result message"),
  }
}

#[test]
fn test_server_message_error_serialization() {
  let msg = ServerMessage::Error {
    id: "123".to_string(),
    error: "Something went wrong".to_string(),
  };

  let json = serde_json::to_string(&msg).unwrap();
  assert!(json.contains("\"type\":\"error\""));
  assert!(json.contains("\"error\":\"Something went wrong\""));
}

#[test]
fn test_server_message_subscribed_serialization() {
  let msg = ServerMessage::Subscribed {
    id: "sub1".to_string(),
  };

  let json = serde_json::to_string(&msg).unwrap();
  assert!(json.contains("\"type\":\"subscribed\""));
}

#[test]
fn test_server_message_pong_serialization() {
  let msg = ServerMessage::Pong {
    id: "ping1".to_string(),
  };

  let json = serde_json::to_string(&msg).unwrap();
  assert!(json.contains("\"type\":\"pong\""));
}

#[test]
fn test_change_event_initial_serialization() {
  let doc = Document {
    id: uuid::Uuid::new_v4(),
    collection: "users".to_string(),
    data: serde_json::json!({"name": "Alice"}),
    created_at: "2024-01-01T00:00:00Z".to_string(),
    updated_at: "2024-01-01T00:00:00Z".to_string(),
  };

  let event = ChangeEvent::Initial { document: doc };
  let json = serde_json::to_string(&event).unwrap();
  assert!(json.contains("\"type\":\"initial\""));
  assert!(json.contains("\"document\":"));
}

#[test]
fn test_change_event_insert_serialization() {
  let doc = Document {
    id: uuid::Uuid::new_v4(),
    collection: "users".to_string(),
    data: serde_json::json!({"name": "Bob"}),
    created_at: "2024-01-01T00:00:00Z".to_string(),
    updated_at: "2024-01-01T00:00:00Z".to_string(),
  };

  let event = ChangeEvent::Insert { new: doc };
  let json = serde_json::to_string(&event).unwrap();
  assert!(json.contains("\"type\":\"insert\""));
  assert!(json.contains("\"new\":"));
}

#[test]
fn test_change_event_update_serialization() {
  let doc = Document {
    id: uuid::Uuid::new_v4(),
    collection: "users".to_string(),
    data: serde_json::json!({"name": "Charlie"}),
    created_at: "2024-01-01T00:00:00Z".to_string(),
    updated_at: "2024-01-02T00:00:00Z".to_string(),
  };

  let event = ChangeEvent::Update {
    old: serde_json::json!({"name": "Bob"}),
    new: doc,
  };
  let json = serde_json::to_string(&event).unwrap();
  assert!(json.contains("\"type\":\"update\""));
  assert!(json.contains("\"old\":"));
  assert!(json.contains("\"new\":"));
}

#[test]
fn test_change_event_delete_serialization() {
  let doc = Document {
    id: uuid::Uuid::new_v4(),
    collection: "users".to_string(),
    data: serde_json::json!({"name": "Alice"}),
    created_at: "2024-01-01T00:00:00Z".to_string(),
    updated_at: "2024-01-01T00:00:00Z".to_string(),
  };

  let event = ChangeEvent::Delete { old: doc };
  let json = serde_json::to_string(&event).unwrap();
  assert!(json.contains("\"type\":\"delete\""));
  assert!(json.contains("\"old\":"));
}

#[test]
fn test_document_serialization() {
  let doc = Document {
    id: uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
    collection: "users".to_string(),
    data: serde_json::json!({"name": "Test", "age": 30}),
    created_at: "2024-01-01T00:00:00Z".to_string(),
    updated_at: "2024-01-01T00:00:00Z".to_string(),
  };

  let json = serde_json::to_string(&doc).unwrap();
  assert!(json.contains("\"id\":\"550e8400-e29b-41d4-a716-446655440000\""));
  assert!(json.contains("\"collection\":\"users\""));
  assert!(json.contains("\"name\":\"Test\""));
  assert!(json.contains("\"age\":30"));

  let parsed: Document = serde_json::from_str(&json).unwrap();
  assert_eq!(parsed.id, doc.id);
  assert_eq!(parsed.collection, doc.collection);
  assert_eq!(parsed.data["name"], "Test");
  assert_eq!(parsed.data["age"], 30);
}

#[test]
fn test_msgpack_client_message_roundtrip() {
  let msg = ClientMessage::Query {
    id: "test123".to_string(),
    query: "db.table(\"test\").run()".to_string(),
  };

  let packed = rmp_serde::to_vec(&msg).unwrap();
  let unpacked: ClientMessage = rmp_serde::from_slice(&packed).unwrap();

  match unpacked {
    ClientMessage::Query { id, query } => {
      assert_eq!(id, "test123");
      assert_eq!(query, "db.table(\"test\").run()");
    }
    _ => panic!("Expected Query"),
  }
}

#[test]
fn test_msgpack_server_message_roundtrip() {
  let msg = ServerMessage::Result {
    id: "resp1".to_string(),
    data: serde_json::json!({"status": "ok", "count": 42}),
  };

  let packed = rmp_serde::to_vec(&msg).unwrap();
  let unpacked: ServerMessage = rmp_serde::from_slice(&packed).unwrap();

  match unpacked {
    ServerMessage::Result { id, data } => {
      assert_eq!(id, "resp1");
      assert_eq!(data["status"], "ok");
      assert_eq!(data["count"], 42);
    }
    _ => panic!("Expected Result"),
  }
}

#[test]
fn test_encoding_default() {
  let encoding = Encoding::default();
  assert_eq!(encoding, Encoding::MessagePack);
}
