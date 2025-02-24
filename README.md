# sovran-typemap

[![Crates.io](https://img.shields.io/crates/v/sovran-typemap.svg)](https://crates.io/crates/sovran-typemap)
[![Documentation](https://docs.rs/sovran-typemap/badge.svg)](https://docs.rs/sovran-typemap)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A thread-safe, type-safe heterogeneous container library for Rust.

`sovran-typemap` provides a flexible way to store different types in a single container while maintaining type-safety through runtime checks. This is particularly useful for applications that need to share state between components without requiring all components to know about all types.

## Key Features

- **Type-safe**: Values are checked at runtime to ensure type correctness
- **Thread-safe**: Built on `Arc<Mutex<_>>` for safe concurrent access
- **Ergonomic API**: Simple methods for storing, retrieving, and modifying values
- **Flexible**: Supports any type that implements `Any + Send + Sync`
- **No macros**: Pure runtime solution without complex macro magic

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
sovran-typemap = "0.1.0"
```

## Basic Usage

```rust
use sovran_typemap::{TypeStore, StoreError};

fn main() -> Result<(), StoreError> {
    // Create a new store with string keys
    let store = TypeStore::<String>::new();

    // Store values of different types
    store.set("number".to_string(), 42i32)?;
    store.set("text".to_string(), "Hello, world!".to_string())?;
    store.set("data".to_string(), vec![1, 2, 3, 4, 5])?;

    // Retrieve values in a type-safe way
    let num = store.get::<i32>(&"number".to_string())?;
    let text = store.get::<String>(&"text".to_string())?;

    println!("Number: {}", num);
    println!("Text: {}", text);

    // Handle errors properly
    match store.get::<bool>(&"nonexistent".to_string()) {
        Ok(value) => println!("Value: {}", value),
        Err(StoreError::KeyNotFound) => println!("Key doesn't exist"),
        Err(StoreError::TypeMismatch) => println!("Type doesn't match"),
        Err(e) => println!("Other error: {}", e),
    }

    Ok(())
}
```

## Using with_mut to Modify Values In-Place

```rust
use sovran_typemap::{TypeStore, StoreError};
use std::collections::HashMap;

fn main() -> Result<(), StoreError> {
    let store = TypeStore::<String>::new();

    // Initialize a counter map
    let mut counters = HashMap::new();
    counters.insert("visits".to_string(), 0);
    store.set("counters".to_string(), counters)?;

    // Update a counter in-place
    store.with_mut(&"counters".to_string(), |counters: &mut HashMap<String, i32>| {
        let visits = counters.entry("visits".to_string()).or_insert(0);
        *visits += 1;
    })?;

    // Add a new counter
    store.with_mut(&"counters".to_string(), |counters: &mut HashMap<String, i32>| {
        counters.insert("api_calls".to_string(), 1);
    })?;

    // Read current values
    let visit_count = store.with(&"counters".to_string(), |counters: &HashMap<String, i32>| {
        counters.get("visits").copied().unwrap_or(0)
    })?;

    println!("Visit count: {}", visit_count);

    Ok(())
}
```

## Sharing State Between Components

```rust
use sovran_typemap::{TypeStore, StoreError};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

struct UserService {
    store: Arc<TypeStore<String>>,
}

struct LogService {
    store: Arc<TypeStore<String>>,
}

impl UserService {
    fn new(store: Arc<TypeStore<String>>) -> Self {
        Self { store }
    }

    fn get_user_count(&self) -> Result<usize, StoreError> {
        self.store.with(&"users".to_string(), |users: &Vec<String>| {
            users.len()
        })
    }

    fn add_user(&self, username: String) -> Result<(), StoreError> {
        // Initialize users vector if it doesn't exist yet
        if !self.store.contains_key(&"users".to_string())? {
            self.store.set("users".to_string(), Vec::<String>::new())?;
        }

        // Add a user
        self.store.with_mut(&"users".to_string(), |users: &mut Vec<String>| {
            users.push(username);
        })
    }
}

impl LogService {
    fn new(store: Arc<TypeStore<String>>) -> Self {
        Self { store }
    }

    fn log(&self, message: String) -> Result<(), StoreError> {
        // Initialize logs if they don't exist
        if !self.store.contains_key(&"logs".to_string())? {
            self.store.set("logs".to_string(), Vec::<String>::new())?;
        }

        // Add log entry with timestamp
        self.store.with_mut(&"logs".to_string(), |logs: &mut Vec<String>| {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            logs.push(format!("[{}] {}", now, message));
        })
    }

    fn get_recent_logs(&self, count: usize) -> Result<Vec<String>, StoreError> {
        self.store.with(&"logs".to_string(), |logs: &Vec<String>| {
            logs.iter()
               .rev()
               .take(count)
               .cloned()
               .collect()
        })
    }
}

fn main() -> Result<(), StoreError> {
    // Create a shared store
    let store = Arc::new(TypeStore::<String>::new());

    // Create services that share the store
    let user_service = UserService::new(Arc::clone(&store));
    let log_service = LogService::new(Arc::clone(&store));

    // Use the services
    user_service.add_user("alice".to_string())?;
    log_service.log("User alice added".to_string())?;

    user_service.add_user("bob".to_string())?;
    log_service.log("User bob added".to_string())?;

    // Get information from both services
    println!("User count: {}", user_service.get_user_count()?);
    
    println!("Recent logs:");
    for log in log_service.get_recent_logs(5)? {
        println!("  {}", log);
    }

    Ok(())
}
```

## Error Handling

The library provides detailed error types to help with error handling:

```rust
use sovran_typemap::{TypeStore, StoreError};

fn main() {
    let store = TypeStore::<String>::new();

    // Set a value for demonstration
    if let Err(e) = store.set("config".to_string(), vec!["setting1", "setting2"]) {
        eprintln!("Failed to store config: {}", e);
        return;
    }

    // Try to get a value with the wrong type
    match store.get::<String>(&"config".to_string()) {
        Ok(value) => println!("Config: {}", value),
        Err(StoreError::KeyNotFound) => println!("Config key not found"),
        Err(StoreError::TypeMismatch) => println!("Config is not a String"),
        Err(StoreError::LockError) => println!("Failed to acquire lock"),
    }

    // Try to access a non-existent key
    match store.get::<i32>(&"settings".to_string()) {
        Ok(value) => println!("Setting: {}", value),
        Err(StoreError::KeyNotFound) => println!("Settings key not found"),
        Err(e) => println!("Other error: {}", e),
    }
}
```

## Available Methods

- `new()` - Create a new empty TypeStore
- `set(key, value)` - Store a value of any type
- `set_with(key, closure)` - Store a value generated by a closure
- `get<T>(key)` - Get a clone of a value
- `with<T, F, R>(key, closure)` - Access a value with a read-only closure
- `with_mut<T, F, R>(key, closure)` - Access a value with a read-write closure
- `remove(key)` - Remove a value
- `contains_key(key)` - Check if a key exists
- `keys()` - Get all keys
- `len()` - Get the number of items
- `is_empty()` - Check if the store is empty

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contribution

Contributions are welcome! Please feel free to submit a Pull Request.