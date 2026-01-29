//! Query builder for SquirrelDB
//!
//! Provides a fluent API for building queries using MongoDB-like naming: find/sort/limit

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fmt;

/// Sort direction
#[derive(Debug, Clone, Copy)]
pub enum SortDir {
    Asc,
    Desc,
}

impl fmt::Display for SortDir {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SortDir::Asc => write!(f, "asc"),
            SortDir::Desc => write!(f, "desc"),
        }
    }
}

/// Sort specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortSpec {
    pub field: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
}

/// Changes subscription options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangesSpec {
    #[serde(rename = "includeInitial", default)]
    pub include_initial: bool,
}

/// Structured query object sent over the wire
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredQuery {
    pub table: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<Vec<SortSpec>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changes: Option<ChangesSpec>,
}

/// Filter condition for queries
#[derive(Debug, Clone)]
pub enum Filter {
    Eq(String, serde_json::Value),
    Ne(String, serde_json::Value),
    Gt(String, f64),
    Gte(String, f64),
    Lt(String, f64),
    Lte(String, f64),
    In(String, Vec<serde_json::Value>),
    NotIn(String, Vec<serde_json::Value>),
    Contains(String, String),
    StartsWith(String, String),
    EndsWith(String, String),
    Exists(String, bool),
    And(Vec<Filter>),
    Or(Vec<Filter>),
    Not(Box<Filter>),
}

impl Filter {
    fn compile(&self) -> String {
        match self {
            Filter::Eq(field, value) => format!("doc.{} === {}", field, value),
            Filter::Ne(field, value) => format!("doc.{} !== {}", field, value),
            Filter::Gt(field, value) => format!("doc.{} > {}", field, value),
            Filter::Gte(field, value) => format!("doc.{} >= {}", field, value),
            Filter::Lt(field, value) => format!("doc.{} < {}", field, value),
            Filter::Lte(field, value) => format!("doc.{} <= {}", field, value),
            Filter::In(field, values) => {
                let arr = serde_json::to_string(values).unwrap_or_else(|_| "[]".to_string());
                format!("{}.includes(doc.{})", arr, field)
            }
            Filter::NotIn(field, values) => {
                let arr = serde_json::to_string(values).unwrap_or_else(|_| "[]".to_string());
                format!("!{}.includes(doc.{})", arr, field)
            }
            Filter::Contains(field, value) => {
                format!("doc.{}.includes({})", field, json!(value))
            }
            Filter::StartsWith(field, value) => {
                format!("doc.{}.startsWith({})", field, json!(value))
            }
            Filter::EndsWith(field, value) => {
                format!("doc.{}.endsWith({})", field, json!(value))
            }
            Filter::Exists(field, value) => {
                if *value {
                    format!("doc.{} !== undefined", field)
                } else {
                    format!("doc.{} === undefined", field)
                }
            }
            Filter::And(conditions) => {
                let parts: Vec<String> = conditions.iter().map(|c| c.compile()).collect();
                format!("({})", parts.join(" && "))
            }
            Filter::Or(conditions) => {
                let parts: Vec<String> = conditions.iter().map(|c| c.compile()).collect();
                format!("({})", parts.join(" || "))
            }
            Filter::Not(condition) => {
                format!("!({})", condition.compile())
            }
        }
    }

    /// Convert filter to structured query format
    fn to_structured(&self) -> HashMap<String, serde_json::Value> {
        let mut result = HashMap::new();
        match self {
            Filter::Eq(field, value) => {
                result.insert(field.clone(), json!({"$eq": value}));
            }
            Filter::Ne(field, value) => {
                result.insert(field.clone(), json!({"$ne": value}));
            }
            Filter::Gt(field, value) => {
                result.insert(field.clone(), json!({"$gt": value}));
            }
            Filter::Gte(field, value) => {
                result.insert(field.clone(), json!({"$gte": value}));
            }
            Filter::Lt(field, value) => {
                result.insert(field.clone(), json!({"$lt": value}));
            }
            Filter::Lte(field, value) => {
                result.insert(field.clone(), json!({"$lte": value}));
            }
            Filter::In(field, values) => {
                result.insert(field.clone(), json!({"$in": values}));
            }
            Filter::NotIn(field, values) => {
                result.insert(field.clone(), json!({"$nin": values}));
            }
            Filter::Contains(field, value) => {
                result.insert(field.clone(), json!({"$contains": value}));
            }
            Filter::StartsWith(field, value) => {
                result.insert(field.clone(), json!({"$startsWith": value}));
            }
            Filter::EndsWith(field, value) => {
                result.insert(field.clone(), json!({"$endsWith": value}));
            }
            Filter::Exists(field, value) => {
                result.insert(field.clone(), json!({"$exists": value}));
            }
            Filter::And(conditions) => {
                let structured: Vec<serde_json::Value> = conditions
                    .iter()
                    .map(|c| serde_json::to_value(c.to_structured()).unwrap_or_default())
                    .collect();
                result.insert("$and".to_string(), json!(structured));
            }
            Filter::Or(conditions) => {
                let structured: Vec<serde_json::Value> = conditions
                    .iter()
                    .map(|c| serde_json::to_value(c.to_structured()).unwrap_or_default())
                    .collect();
                result.insert("$or".to_string(), json!(structured));
            }
            Filter::Not(condition) => {
                result.insert(
                    "$not".to_string(),
                    serde_json::to_value(condition.to_structured()).unwrap_or_default(),
                );
            }
        }
        result
    }
}

/// Field expression builder for fluent filter construction
pub struct Field {
    name: String,
}

impl Field {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    pub fn eq(self, value: impl Into<serde_json::Value>) -> Filter {
        Filter::Eq(self.name, value.into())
    }

    pub fn ne(self, value: impl Into<serde_json::Value>) -> Filter {
        Filter::Ne(self.name, value.into())
    }

    pub fn gt(self, value: f64) -> Filter {
        Filter::Gt(self.name, value)
    }

    pub fn gte(self, value: f64) -> Filter {
        Filter::Gte(self.name, value)
    }

    pub fn lt(self, value: f64) -> Filter {
        Filter::Lt(self.name, value)
    }

    pub fn lte(self, value: f64) -> Filter {
        Filter::Lte(self.name, value)
    }

    pub fn is_in(self, values: Vec<serde_json::Value>) -> Filter {
        Filter::In(self.name, values)
    }

    pub fn not_in(self, values: Vec<serde_json::Value>) -> Filter {
        Filter::NotIn(self.name, values)
    }

    pub fn contains(self, value: impl Into<String>) -> Filter {
        Filter::Contains(self.name, value.into())
    }

    pub fn starts_with(self, value: impl Into<String>) -> Filter {
        Filter::StartsWith(self.name, value.into())
    }

    pub fn ends_with(self, value: impl Into<String>) -> Filter {
        Filter::EndsWith(self.name, value.into())
    }

    pub fn exists(self, value: bool) -> Filter {
        Filter::Exists(self.name, value)
    }
}

/// Create a field expression
pub fn field(name: impl Into<String>) -> Field {
    Field::new(name)
}

/// Combine filters with AND
pub fn and(filters: Vec<Filter>) -> Filter {
    Filter::And(filters)
}

/// Combine filters with OR
pub fn or(filters: Vec<Filter>) -> Filter {
    Filter::Or(filters)
}

/// Negate a filter
pub fn not(filter: Filter) -> Filter {
    Filter::Not(Box::new(filter))
}

/// Query builder for SquirrelDB
///
/// Uses MongoDB-like naming: find/sort/limit
///
/// # Example
/// ```
/// use squirreldb::query::{QueryBuilder, field, SortDir};
///
/// let query = QueryBuilder::table("users")
///     .find(field("age").gt(21.0))
///     .sort("name", SortDir::Asc)
///     .limit(10)
///     .compile();
/// ```
pub struct QueryBuilder {
    table_name: String,
    filter: Option<Filter>,
    sort_specs: Vec<SortSpec>,
    limit_value: Option<usize>,
    skip_value: Option<usize>,
    is_changes: bool,
}

impl QueryBuilder {
    /// Create a new query builder for a table
    pub fn table(name: impl Into<String>) -> Self {
        Self {
            table_name: name.into(),
            filter: None,
            sort_specs: Vec::new(),
            limit_value: None,
            skip_value: None,
            is_changes: false,
        }
    }

    /// Add a filter condition
    pub fn find(mut self, filter: Filter) -> Self {
        self.filter = Some(filter);
        self
    }

    /// Sort by field
    pub fn sort(mut self, field: impl Into<String>, direction: SortDir) -> Self {
        self.sort_specs.push(SortSpec {
            field: field.into(),
            direction: Some(direction.to_string()),
        });
        self
    }

    /// Limit number of results
    pub fn limit(mut self, n: usize) -> Self {
        self.limit_value = Some(n);
        self
    }

    /// Skip results (offset)
    pub fn skip(mut self, n: usize) -> Self {
        self.skip_value = Some(n);
        self
    }

    /// Subscribe to changes
    pub fn changes(mut self) -> Self {
        self.is_changes = true;
        self
    }

    /// Compile to SquirrelDB JS query string (legacy)
    pub fn compile(&self) -> String {
        let mut query = format!(r#"db.table("{}")"#, self.table_name);

        if let Some(ref filter) = self.filter {
            query.push_str(&format!(".filter(doc => {})", filter.compile()));
        }

        for spec in &self.sort_specs {
            match spec.direction.as_deref() {
                Some("desc") => {
                    query.push_str(&format!(r#".orderBy("{}", "desc")"#, spec.field));
                }
                _ => {
                    query.push_str(&format!(r#".orderBy("{}")"#, spec.field));
                }
            }
        }

        if let Some(limit) = self.limit_value {
            query.push_str(&format!(".limit({})", limit));
        }

        if let Some(skip) = self.skip_value {
            query.push_str(&format!(".skip({})", skip));
        }

        if self.is_changes {
            query.push_str(".changes()");
        } else {
            query.push_str(".run()");
        }

        query
    }

    /// Compile to structured query object (preferred, no JS evaluation on server)
    pub fn compile_structured(&self) -> StructuredQuery {
        let filter = self.filter.as_ref().map(|f| f.to_structured());

        let sort = if self.sort_specs.is_empty() {
            None
        } else {
            Some(
                self.sort_specs
                    .iter()
                    .map(|s| SortSpec {
                        field: s.field.clone(),
                        direction: s.direction.clone(),
                    })
                    .collect(),
            )
        };

        let changes = if self.is_changes {
            Some(ChangesSpec {
                include_initial: false,
            })
        } else {
            None
        };

        StructuredQuery {
            table: self.table_name.clone(),
            filter,
            sort,
            limit: self.limit_value,
            skip: self.skip_value,
            changes,
        }
    }
}

impl fmt::Display for QueryBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.compile())
    }
}

/// Create a table query builder
pub fn table(name: impl Into<String>) -> QueryBuilder {
    QueryBuilder::table(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_query() {
        let query = table("users").compile();
        assert_eq!(query, r#"db.table("users").run()"#);
    }

    #[test]
    fn test_filter_query() {
        let query = table("users").find(field("age").gt(21.0)).compile();
        assert_eq!(query, r#"db.table("users").filter(doc => doc.age > 21).run()"#);
    }

    #[test]
    fn test_full_query() {
        let query = table("users")
            .find(field("age").gt(21.0))
            .sort("name", SortDir::Asc)
            .limit(10)
            .skip(5)
            .compile();
        assert_eq!(
            query,
            r#"db.table("users").filter(doc => doc.age > 21).orderBy("name").limit(10).skip(5).run()"#
        );
    }

    #[test]
    fn test_and_filter() {
        let query = table("users")
            .find(and(vec![field("age").gt(21.0), field("active").eq(true)]))
            .compile();
        assert!(query.contains("&&"));
    }
}
