//! Redis-compatible async cache client using RESP protocol
//!
//! Provides a TCP-based cache client that speaks the Redis Serialization Protocol (RESP).

use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

/// Cache client options
pub struct CacheOptions {
    pub host: String,
    pub port: u16,
}

impl Default for CacheOptions {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 6379,
        }
    }
}

/// Cache error types
#[derive(Debug)]
pub enum CacheError {
    Connection(String),
    Io(std::io::Error),
    Protocol(String),
    Server(String),
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheError::Connection(msg) => write!(f, "Connection error: {}", msg),
            CacheError::Io(e) => write!(f, "IO error: {}", e),
            CacheError::Protocol(msg) => write!(f, "Protocol error: {}", msg),
            CacheError::Server(msg) => write!(f, "Server error: {}", msg),
        }
    }
}

impl std::error::Error for CacheError {}

impl From<std::io::Error> for CacheError {
    fn from(e: std::io::Error) -> Self {
        CacheError::Io(e)
    }
}

/// RESP value types
#[derive(Debug, Clone)]
pub enum RespValue {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(Option<String>),
    Array(Option<Vec<RespValue>>),
}

impl RespValue {
    fn as_string(&self) -> Option<String> {
        match self {
            RespValue::SimpleString(s) => Some(s.clone()),
            RespValue::BulkString(Some(s)) => Some(s.clone()),
            _ => None,
        }
    }

    fn as_integer(&self) -> Option<i64> {
        match self {
            RespValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    fn as_array(&self) -> Option<&Vec<RespValue>> {
        match self {
            RespValue::Array(Some(arr)) => Some(arr),
            _ => None,
        }
    }

    fn is_ok(&self) -> bool {
        matches!(self, RespValue::SimpleString(s) if s == "OK")
    }
}

/// Encode a RESP command
fn encode_command(args: &[&str]) -> Vec<u8> {
    let mut buf = Vec::new();

    // Array header
    buf.extend_from_slice(format!("*{}\r\n", args.len()).as_bytes());

    // Bulk strings for each argument
    for arg in args {
        buf.extend_from_slice(format!("${}\r\n", arg.len()).as_bytes());
        buf.extend_from_slice(arg.as_bytes());
        buf.extend_from_slice(b"\r\n");
    }

    buf
}

/// Parse a RESP response from a buffered reader
async fn parse_resp<R: AsyncBufReadExt + Unpin>(reader: &mut R) -> Result<RespValue, CacheError> {
    let mut line = String::new();
    reader.read_line(&mut line).await?;

    if line.is_empty() {
        return Err(CacheError::Protocol("Empty response".to_string()));
    }

    let line = line.trim_end_matches("\r\n").trim_end_matches('\n');

    if line.is_empty() {
        return Err(CacheError::Protocol("Empty response line".to_string()));
    }

    let prefix = line.chars().next().unwrap();
    let content = &line[1..];

    match prefix {
        '+' => Ok(RespValue::SimpleString(content.to_string())),
        '-' => Ok(RespValue::Error(content.to_string())),
        ':' => {
            let i = content
                .parse::<i64>()
                .map_err(|_| CacheError::Protocol(format!("Invalid integer: {}", content)))?;
            Ok(RespValue::Integer(i))
        }
        '$' => {
            let len = content.parse::<i64>().map_err(|_| {
                CacheError::Protocol(format!("Invalid bulk string length: {}", content))
            })?;

            if len < 0 {
                return Ok(RespValue::BulkString(None));
            }

            let len = len as usize;
            let mut data = vec![0u8; len];
            reader.read_exact(&mut data).await?;

            // Read trailing \r\n
            let mut crlf = [0u8; 2];
            reader.read_exact(&mut crlf).await?;

            let s = String::from_utf8(data)
                .map_err(|_| CacheError::Protocol("Invalid UTF-8 in bulk string".to_string()))?;

            Ok(RespValue::BulkString(Some(s)))
        }
        '*' => {
            let count = content
                .parse::<i64>()
                .map_err(|_| CacheError::Protocol(format!("Invalid array length: {}", content)))?;

            if count < 0 {
                return Ok(RespValue::Array(None));
            }

            let count = count as usize;
            let mut items = Vec::with_capacity(count);

            for _ in 0..count {
                items.push(Box::pin(parse_resp(reader)).await?);
            }

            Ok(RespValue::Array(Some(items)))
        }
        _ => Err(CacheError::Protocol(format!(
            "Unknown RESP prefix: {}",
            prefix
        ))),
    }
}

/// Redis-compatible async cache client
pub struct CacheClient {
    stream: BufReader<TcpStream>,
}

impl CacheClient {
    /// Connect to a cache server
    pub async fn connect(opts: Option<CacheOptions>) -> Result<Self, CacheError> {
        let opts = opts.unwrap_or_default();
        let addr = format!("{}:{}", opts.host, opts.port);

        let stream = TcpStream::connect(&addr)
            .await
            .map_err(|e| CacheError::Connection(format!("Failed to connect to {}: {}", addr, e)))?;

        Ok(Self {
            stream: BufReader::new(stream),
        })
    }

    /// Send a command and receive the response
    async fn command(&mut self, args: &[&str]) -> Result<RespValue, CacheError> {
        let cmd = encode_command(args);
        self.stream.get_mut().write_all(&cmd).await?;
        self.stream.get_mut().flush().await?;

        let resp = parse_resp(&mut self.stream).await?;

        if let RespValue::Error(msg) = &resp {
            return Err(CacheError::Server(msg.clone()));
        }

        Ok(resp)
    }

    /// Get a value by key
    pub async fn get(&mut self, key: &str) -> Result<Option<String>, CacheError> {
        let resp = self.command(&["GET", key]).await?;
        Ok(resp.as_string())
    }

    /// Set a value with optional TTL in seconds
    pub async fn set(
        &mut self,
        key: &str,
        value: &str,
        ttl: Option<u64>,
    ) -> Result<(), CacheError> {
        let resp = match ttl {
            Some(seconds) => {
                let ttl_str = seconds.to_string();
                self.command(&["SET", key, value, "EX", &ttl_str]).await?
            }
            None => self.command(&["SET", key, value]).await?,
        };

        if resp.is_ok() {
            Ok(())
        } else {
            Err(CacheError::Protocol("SET did not return OK".to_string()))
        }
    }

    /// Delete a key, returns true if key existed
    pub async fn del(&mut self, key: &str) -> Result<bool, CacheError> {
        let resp = self.command(&["DEL", key]).await?;
        Ok(resp.as_integer().unwrap_or(0) > 0)
    }

    /// Check if a key exists
    pub async fn exists(&mut self, key: &str) -> Result<bool, CacheError> {
        let resp = self.command(&["EXISTS", key]).await?;
        Ok(resp.as_integer().unwrap_or(0) > 0)
    }

    /// Set expiration on a key
    pub async fn expire(&mut self, key: &str, seconds: u64) -> Result<bool, CacheError> {
        let ttl_str = seconds.to_string();
        let resp = self.command(&["EXPIRE", key, &ttl_str]).await?;
        Ok(resp.as_integer().unwrap_or(0) > 0)
    }

    /// Get TTL of a key in seconds (-1 = no expiry, -2 = key doesn't exist)
    pub async fn ttl(&mut self, key: &str) -> Result<i64, CacheError> {
        let resp = self.command(&["TTL", key]).await?;
        Ok(resp.as_integer().unwrap_or(-2))
    }

    /// Remove expiration from a key
    pub async fn persist(&mut self, key: &str) -> Result<bool, CacheError> {
        let resp = self.command(&["PERSIST", key]).await?;
        Ok(resp.as_integer().unwrap_or(0) > 0)
    }

    /// Increment a key's integer value by 1
    pub async fn incr(&mut self, key: &str) -> Result<i64, CacheError> {
        let resp = self.command(&["INCR", key]).await?;
        resp.as_integer()
            .ok_or_else(|| CacheError::Protocol("INCR did not return integer".to_string()))
    }

    /// Decrement a key's integer value by 1
    pub async fn decr(&mut self, key: &str) -> Result<i64, CacheError> {
        let resp = self.command(&["DECR", key]).await?;
        resp.as_integer()
            .ok_or_else(|| CacheError::Protocol("DECR did not return integer".to_string()))
    }

    /// Increment a key's integer value by amount
    pub async fn incrby(&mut self, key: &str, amount: i64) -> Result<i64, CacheError> {
        let amount_str = amount.to_string();
        let resp = self.command(&["INCRBY", key, &amount_str]).await?;
        resp.as_integer()
            .ok_or_else(|| CacheError::Protocol("INCRBY did not return integer".to_string()))
    }

    /// Get all keys matching a pattern
    pub async fn keys(&mut self, pattern: &str) -> Result<Vec<String>, CacheError> {
        let resp = self.command(&["KEYS", pattern]).await?;
        match resp.as_array() {
            Some(arr) => Ok(arr.iter().filter_map(|v| v.as_string()).collect()),
            None => Ok(Vec::new()),
        }
    }

    /// Get multiple values at once
    pub async fn mget(&mut self, keys: &[&str]) -> Result<Vec<Option<String>>, CacheError> {
        let mut args = vec!["MGET"];
        args.extend(keys);

        let resp = self.command(&args).await?;

        match resp.as_array() {
            Some(arr) => Ok(arr
                .iter()
                .map(|v| match v {
                    RespValue::BulkString(s) => s.clone(),
                    RespValue::SimpleString(s) => Some(s.clone()),
                    _ => None,
                })
                .collect()),
            None => Ok(vec![None; keys.len()]),
        }
    }

    /// Set multiple key-value pairs at once
    pub async fn mset(&mut self, pairs: &[(&str, &str)]) -> Result<(), CacheError> {
        let mut args = vec!["MSET"];
        for (k, v) in pairs {
            args.push(k);
            args.push(v);
        }

        let resp = self.command(&args).await?;

        if resp.is_ok() {
            Ok(())
        } else {
            Err(CacheError::Protocol("MSET did not return OK".to_string()))
        }
    }

    /// Get number of keys in the database
    pub async fn dbsize(&mut self) -> Result<i64, CacheError> {
        let resp = self.command(&["DBSIZE"]).await?;
        resp.as_integer()
            .ok_or_else(|| CacheError::Protocol("DBSIZE did not return integer".to_string()))
    }

    /// Delete all keys in the current database
    pub async fn flush(&mut self) -> Result<(), CacheError> {
        let resp = self.command(&["FLUSHDB"]).await?;

        if resp.is_ok() {
            Ok(())
        } else {
            Err(CacheError::Protocol(
                "FLUSHDB did not return OK".to_string(),
            ))
        }
    }

    /// Get server information
    pub async fn info(&mut self) -> Result<HashMap<String, String>, CacheError> {
        let resp = self.command(&["INFO"]).await?;

        let text = resp
            .as_string()
            .ok_or_else(|| CacheError::Protocol("INFO did not return string".to_string()))?;

        let mut result = HashMap::new();

        for line in text.lines() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once(':') {
                result.insert(key.to_string(), value.to_string());
            }
        }

        Ok(result)
    }

    /// Ping the server
    pub async fn ping(&mut self) -> Result<(), CacheError> {
        let resp = self.command(&["PING"]).await?;

        match &resp {
            RespValue::SimpleString(s) if s == "PONG" => Ok(()),
            _ => Err(CacheError::Protocol("PING did not return PONG".to_string())),
        }
    }

    /// Close the connection
    pub async fn close(&mut self) -> Result<(), CacheError> {
        // Send QUIT command
        let _ = self.command(&["QUIT"]).await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_command() {
        let cmd = encode_command(&["GET", "foo"]);
        assert_eq!(cmd, b"*2\r\n$3\r\nGET\r\n$3\r\nfoo\r\n");
    }

    #[test]
    fn test_encode_set_command() {
        let cmd = encode_command(&["SET", "key", "value"]);
        assert_eq!(cmd, b"*3\r\n$3\r\nSET\r\n$3\r\nkey\r\n$5\r\nvalue\r\n");
    }

    #[test]
    fn test_default_options() {
        let opts = CacheOptions::default();
        assert_eq!(opts.host, "localhost");
        assert_eq!(opts.port, 6379);
    }

    #[tokio::test]
    async fn test_parse_simple_string() {
        let data = b"+OK\r\n";
        let mut reader = BufReader::new(&data[..]);
        let resp = parse_resp(&mut reader).await.unwrap();
        assert!(matches!(resp, RespValue::SimpleString(s) if s == "OK"));
    }

    #[tokio::test]
    async fn test_parse_error() {
        let data = b"-ERR unknown command\r\n";
        let mut reader = BufReader::new(&data[..]);
        let resp = parse_resp(&mut reader).await.unwrap();
        assert!(matches!(resp, RespValue::Error(s) if s == "ERR unknown command"));
    }

    #[tokio::test]
    async fn test_parse_integer() {
        let data = b":42\r\n";
        let mut reader = BufReader::new(&data[..]);
        let resp = parse_resp(&mut reader).await.unwrap();
        assert!(matches!(resp, RespValue::Integer(42)));
    }

    #[tokio::test]
    async fn test_parse_bulk_string() {
        let data = b"$5\r\nhello\r\n";
        let mut reader = BufReader::new(&data[..]);
        let resp = parse_resp(&mut reader).await.unwrap();
        assert!(matches!(resp, RespValue::BulkString(Some(s)) if s == "hello"));
    }

    #[tokio::test]
    async fn test_parse_null_bulk_string() {
        let data = b"$-1\r\n";
        let mut reader = BufReader::new(&data[..]);
        let resp = parse_resp(&mut reader).await.unwrap();
        assert!(matches!(resp, RespValue::BulkString(None)));
    }

    #[tokio::test]
    async fn test_parse_array() {
        let data = b"*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n";
        let mut reader = BufReader::new(&data[..]);
        let resp = parse_resp(&mut reader).await.unwrap();

        if let RespValue::Array(Some(arr)) = resp {
            assert_eq!(arr.len(), 2);
            assert!(matches!(&arr[0], RespValue::BulkString(Some(s)) if s == "foo"));
            assert!(matches!(&arr[1], RespValue::BulkString(Some(s)) if s == "bar"));
        } else {
            panic!("Expected array");
        }
    }
}
