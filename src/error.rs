use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
pub enum DynamodeError {
    Serialization(String),
    Deserialization(String),
    DynamoDb(String),
    NotFound,
    InvalidKey,
    Validation(String),
    Network(String),
}

impl fmt::Display for DynamodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DynamodeError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            DynamodeError::Deserialization(msg) => write!(f, "Deserialization error: {}", msg),
            DynamodeError::DynamoDb(msg) => write!(f, "DynamoDB error: {}", msg),
            DynamodeError::NotFound => write!(f, "Entity not found"),
            DynamodeError::InvalidKey => write!(f, "Invalid key"),
            DynamodeError::Validation(msg) => write!(f, "Validation error: {}", msg),
            DynamodeError::Network(msg) => write!(f, "Network error: {}", msg),
        }
    }
}

impl StdError for DynamodeError {}
pub type Result<T> = std::result::Result<T, DynamodeError>;
