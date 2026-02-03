//! SquirrelDB Rust SDK - Protocol Tests

use serde_json::json;

#[test]
fn test_ping_message() {
    let msg = json!({"type": "Ping"});
    assert_eq!(msg["type"], "Ping");
}

#[test]
fn test_query_message() {
    let msg = json!({
        "type": "Query",
        "id": "req-123",
        "query": r#"db.table("users").run()"#
    });
    assert_eq!(msg["type"], "Query");
    assert_eq!(msg["id"], "req-123");
    assert!(msg["query"].as_str().unwrap().contains("users"));
}

#[test]
fn test_insert_message() {
    let msg = json!({
        "type": "Insert",
        "id": "req-456",
        "collection": "users",
        "data": {"name": "Alice"}
    });
    assert_eq!(msg["type"], "Insert");
    assert_eq!(msg["collection"], "users");
    assert_eq!(msg["data"]["name"], "Alice");
}

#[test]
fn test_update_message() {
    let msg = json!({
        "type": "Update",
        "id": "req-789",
        "collection": "users",
        "document_id": "doc-123",
        "data": {"name": "Bob"}
    });
    assert_eq!(msg["type"], "Update");
    assert_eq!(msg["document_id"], "doc-123");
}

#[test]
fn test_delete_message() {
    let msg = json!({
        "type": "Delete",
        "id": "req-101",
        "collection": "users",
        "document_id": "doc-123"
    });
    assert_eq!(msg["type"], "Delete");
    assert_eq!(msg["document_id"], "doc-123");
}

#[test]
fn test_subscribe_message() {
    let msg = json!({
        "type": "Subscribe",
        "id": "req-202",
        "query": r#"db.table("users").changes()"#
    });
    assert_eq!(msg["type"], "Subscribe");
    assert!(msg["query"].as_str().unwrap().contains("changes"));
}

#[test]
fn test_unsubscribe_message() {
    let msg = json!({
        "type": "Unsubscribe",
        "id": "req-303",
        "subscription_id": "sub-123"
    });
    assert_eq!(msg["type"], "Unsubscribe");
    assert_eq!(msg["subscription_id"], "sub-123");
}

#[test]
fn test_pong_response() {
    let response = json!({"type": "Pong"});
    assert_eq!(response["type"], "Pong");
}

#[test]
fn test_result_response() {
    let response = json!({
        "type": "Result",
        "id": "req-123",
        "documents": [
            {"id": "1", "collection": "users", "data": {"name": "Alice"}, "created_at": "", "updated_at": ""}
        ]
    });
    assert_eq!(response["type"], "Result");
    assert_eq!(response["documents"].as_array().unwrap().len(), 1);
}

#[test]
fn test_error_response() {
    let response = json!({
        "type": "Error",
        "id": "req-123",
        "message": "Query failed"
    });
    assert_eq!(response["type"], "Error");
    assert_eq!(response["message"], "Query failed");
}

#[test]
fn test_subscribed_response() {
    let response = json!({
        "type": "Subscribed",
        "id": "req-123",
        "subscription_id": "sub-456"
    });
    assert_eq!(response["type"], "Subscribed");
    assert_eq!(response["subscription_id"], "sub-456");
}

#[test]
fn test_change_response() {
    let response = json!({
        "type": "Change",
        "subscription_id": "sub-456",
        "change": {
            "type": "insert",
            "new": {"id": "1", "collection": "users", "data": {}, "created_at": "", "updated_at": ""}
        }
    });
    assert_eq!(response["type"], "Change");
    assert_eq!(response["change"]["type"], "insert");
}

#[test]
fn test_resp_simple_string_format() {
    let response = "+OK\r\n";
    assert!(response.starts_with('+'));
    assert!(response.contains("\r\n"));
}

#[test]
fn test_resp_error_format() {
    let response = "-ERR unknown command\r\n";
    assert!(response.starts_with('-'));
}

#[test]
fn test_resp_integer_format() {
    let response = ":1000\r\n";
    assert!(response.starts_with(':'));
    let value: i64 = response[1..response.find("\r\n").unwrap()].parse().unwrap();
    assert_eq!(value, 1000);
}

#[test]
fn test_resp_bulk_string_format() {
    let value = "hello";
    let response = format!("${}\r\n{}\r\n", value.len(), value);
    assert_eq!(response, "$5\r\nhello\r\n");
}

#[test]
fn test_resp_null_bulk_string_format() {
    let response = "$-1\r\n";
    assert_eq!(response, "$-1\r\n");
}

#[test]
fn test_resp_array_format() {
    let response = "*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n";
    assert!(response.starts_with("*2"));
}
