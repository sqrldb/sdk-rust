//! SquirrelDB Rust SDK - Query Builder Tests

use squirreldb_sdk::{
    field, table, and, or, not,
    SortDirection, ChangesOptions,
};
use serde_json::json;

#[test]
fn test_field_eq() {
    let cond = field("age").eq(25);
    assert_eq!(cond.field, "age");
    assert_eq!(cond.operator, "$eq");
    assert_eq!(cond.value, json!(25));
}

#[test]
fn test_field_ne() {
    let cond = field("status").ne("inactive");
    assert_eq!(cond.operator, "$ne");
    assert_eq!(cond.value, json!("inactive"));
}

#[test]
fn test_field_gt() {
    let cond = field("price").gt(100);
    assert_eq!(cond.operator, "$gt");
    assert_eq!(cond.value, json!(100));
}

#[test]
fn test_field_gte() {
    let cond = field("count").gte(10);
    assert_eq!(cond.operator, "$gte");
}

#[test]
fn test_field_lt() {
    let cond = field("age").lt(18);
    assert_eq!(cond.operator, "$lt");
}

#[test]
fn test_field_lte() {
    let cond = field("rating").lte(5);
    assert_eq!(cond.operator, "$lte");
}

#[test]
fn test_field_is_in() {
    let cond = field("role").is_in(vec![json!("admin"), json!("mod")]);
    assert_eq!(cond.operator, "$in");
    assert!(cond.value.is_array());
}

#[test]
fn test_field_not_in() {
    let cond = field("status").not_in(vec![json!("banned"), json!("deleted")]);
    assert_eq!(cond.operator, "$nin");
}

#[test]
fn test_field_contains() {
    let cond = field("name").contains("test");
    assert_eq!(cond.operator, "$contains");
    assert_eq!(cond.value, json!("test"));
}

#[test]
fn test_field_starts_with() {
    let cond = field("email").starts_with("admin");
    assert_eq!(cond.operator, "$startsWith");
}

#[test]
fn test_field_ends_with() {
    let cond = field("email").ends_with(".com");
    assert_eq!(cond.operator, "$endsWith");
}

#[test]
fn test_field_exists() {
    let cond = field("avatar").exists(true);
    assert_eq!(cond.operator, "$exists");
    assert_eq!(cond.value, json!(true));
}

#[test]
fn test_field_exists_false() {
    let cond = field("deleted_at").exists(false);
    assert_eq!(cond.value, json!(false));
}

#[test]
fn test_table_creates_query_builder() {
    let query = table("users");
    let result = query.compile_structured();
    assert_eq!(result.table, "users");
}

#[test]
fn test_compile_minimal_query() {
    let result = table("users").compile_structured();
    assert_eq!(result.table, "users");
    assert!(result.filter.is_none());
}

#[test]
fn test_find_adds_filter() {
    let result = table("users")
        .find(field("age").gt(21))
        .compile_structured();

    assert_eq!(result.table, "users");
    assert!(result.filter.is_some());
    let filter = result.filter.unwrap();
    assert!(filter.contains_key("age"));
    assert_eq!(filter["age"]["$gt"], json!(21));
}

#[test]
fn test_multiple_filters() {
    let result = table("users")
        .find(field("age").gte(18))
        .find(field("age").lte(65))
        .compile_structured();

    let filter = result.filter.unwrap();
    assert_eq!(filter["age"]["$gte"], json!(18));
    assert_eq!(filter["age"]["$lte"], json!(65));
}

#[test]
fn test_sort_adds_sort_specification() {
    let result = table("users")
        .sort("name", SortDirection::Asc)
        .compile_structured();

    assert!(result.sort.is_some());
    let sorts = result.sort.unwrap();
    assert_eq!(sorts.len(), 1);
    assert_eq!(sorts[0].field, "name");
    assert_eq!(sorts[0].direction, SortDirection::Asc);
}

#[test]
fn test_sort_desc() {
    let result = table("users")
        .sort("created_at", SortDirection::Desc)
        .compile_structured();

    let sorts = result.sort.unwrap();
    assert_eq!(sorts[0].direction, SortDirection::Desc);
}

#[test]
fn test_multiple_sorts() {
    let result = table("posts")
        .sort("pinned", SortDirection::Desc)
        .sort("created_at", SortDirection::Desc)
        .compile_structured();

    let sorts = result.sort.unwrap();
    assert_eq!(sorts.len(), 2);
    assert_eq!(sorts[0].field, "pinned");
    assert_eq!(sorts[1].field, "created_at");
}

#[test]
fn test_limit_sets_max_results() {
    let result = table("users")
        .limit(10)
        .compile_structured();

    assert_eq!(result.limit, Some(10));
}

#[test]
fn test_skip_sets_offset() {
    let result = table("users")
        .skip(20)
        .compile_structured();

    assert_eq!(result.skip, Some(20));
}

#[test]
fn test_changes_enables_subscription() {
    let result = table("messages")
        .changes(None)
        .compile_structured();

    assert!(result.changes.is_some());
    assert!(result.changes.unwrap().include_initial);
}

#[test]
fn test_changes_with_options() {
    let result = table("messages")
        .changes(Some(ChangesOptions { include_initial: false }))
        .compile_structured();

    assert!(!result.changes.unwrap().include_initial);
}

#[test]
fn test_full_query() {
    let result = table("users")
        .find(field("age").gte(18))
        .find(field("status").eq("active"))
        .sort("name", SortDirection::Asc)
        .limit(50)
        .skip(100)
        .compile_structured();

    assert_eq!(result.table, "users");
    let filter = result.filter.unwrap();
    assert_eq!(filter["age"]["$gte"], json!(18));
    assert_eq!(filter["status"]["$eq"], json!("active"));
    assert_eq!(result.limit, Some(50));
    assert_eq!(result.skip, Some(100));
}

#[test]
fn test_compile_returns_json_string() {
    let result = table("users").limit(10).compile().unwrap();
    assert!(result.contains("\"table\":\"users\""));
    assert!(result.contains("\"limit\":10"));
}

#[test]
fn test_and_combines_conditions() {
    let cond = and(vec![
        field("age").gte(18),
        field("active").eq(true),
    ]);

    assert_eq!(cond.field, "$and");
    assert_eq!(cond.operator, "$and");
}

#[test]
fn test_or_combines_conditions() {
    let cond = or(vec![
        field("role").eq("admin"),
        field("role").eq("moderator"),
    ]);

    assert_eq!(cond.field, "$or");
}

#[test]
fn test_not_negates_condition() {
    let cond = not(field("banned").eq(true));

    assert_eq!(cond.field, "$not");
}
