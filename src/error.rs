use std::fmt;

/// Errors that can occur when using TypeMap
#[derive(Debug)]
pub enum MapError {
    /// Failed to acquire lock on the store
    LockError,
    /// The requested key was not found
    KeyNotFound(String),
    /// Attempted to access a value with a type that doesn't match what was stored
    TypeMismatch,
}

impl fmt::Display for MapError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MapError::LockError => write!(f, "Failed to acquire lock"),
            MapError::KeyNotFound(key) => write!(f, "Key not found in store: {}", key),
            MapError::TypeMismatch => write!(f, "Type mismatch for the requested key"),
        }
    }
}

impl std::error::Error for MapError {}
