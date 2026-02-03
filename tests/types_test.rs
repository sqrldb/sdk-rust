//! SquirrelDB Rust SDK - Types Tests

use squirreldb_sdk::{Document, ChangeEvent, Bucket, StorageObject};
use serde_json::json;
use uuid::Uuid;
use chrono::Utc;

#[test]
fn test_document_from_json() {
    let id = Uuid::new_v4();
    let data = json!({
        "id": id.to_string(),
        "collection": "users",
        "data": {"name": "Test"},
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z"
    });

    let doc: Document = serde_json::from_value(data).unwrap();
    assert_eq!(doc.id, id);
    assert_eq!(doc.collection, "users");
    assert_eq!(doc.created_at, "2024-01-01T00:00:00Z");
}

#[test]
fn test_document_to_json() {
    let id = Uuid::new_v4();
    let doc = Document {
        id,
        collection: "test-collection".to_string(),
        data: json!({"foo": "bar"}),
        created_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-01T00:00:00Z".to_string(),
    };

    let json = serde_json::to_value(&doc).unwrap();
    assert_eq!(json["id"], id.to_string());
    assert_eq!(json["collection"], "test-collection");
}

#[test]
fn test_change_event_initial() {
    let id = Uuid::new_v4();
    let data = json!({
        "type": "initial",
        "document": {
            "id": id.to_string(),
            "collection": "users",
            "data": {"name": "Test"},
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z"
        }
    });

    let event: ChangeEvent = serde_json::from_value(data).unwrap();
    assert!(matches!(event, ChangeEvent::Initial { .. }));
}

#[test]
fn test_change_event_insert() {
    let id = Uuid::new_v4();
    let data = json!({
        "type": "insert",
        "new": {
            "id": id.to_string(),
            "collection": "users",
            "data": {"name": "Test"},
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z"
        }
    });

    let event: ChangeEvent = serde_json::from_value(data).unwrap();
    assert!(matches!(event, ChangeEvent::Insert { .. }));
}

#[test]
fn test_bucket_structure() {
    let bucket = Bucket {
        name: "my-bucket".to_string(),
        created_at: Utc::now(),
    };

    assert_eq!(bucket.name, "my-bucket");
}

#[test]
fn test_storage_object_structure() {
    let obj = StorageObject {
        key: "path/to/file.txt".to_string(),
        size: 1024,
        etag: "d41d8cd98f00b204e9800998ecf8427e".to_string(),
        last_modified: Utc::now(),
        content_type: Some("text/plain".to_string()),
    };

    assert_eq!(obj.key, "path/to/file.txt");
    assert_eq!(obj.size, 1024);
    assert_eq!(obj.content_type, Some("text/plain".to_string()));
}

#[test]
fn test_storage_object_null_content_type() {
    let obj = StorageObject {
        key: "file.bin".to_string(),
        size: 2048,
        etag: "abc123".to_string(),
        last_modified: Utc::now(),
        content_type: None,
    };

    assert!(obj.content_type.is_none());
}
