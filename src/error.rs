use std::fmt;

/// Errors that can occur when using TypeStore
#[derive(Debug)]
pub enum StoreError {
    /// Failed to acquire lock on the store
    LockError,
    /// The requested key was not found
    KeyNotFound,
    /// Attempted to access a value with a type that doesn't match what was stored
    TypeMismatch,
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StoreError::LockError => write!(f, "Failed to acquire lock"),
            StoreError::KeyNotFound => write!(f, "Key not found in store"),
            StoreError::TypeMismatch => write!(f, "Type mismatch for the requested key"),
        }
    }
}

impl std::error::Error for StoreError {}
