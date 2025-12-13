# sovran-typemap

[![Crates.io](https://img.shields.io/crates/v/sovran-typemap.svg)](https://crates.io/crates/sovran-typemap)
[![Documentation](https://docs.rs/sovran-typemap/badge.svg)](https://docs.rs/sovran-typemap)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A thread-safe, type-safe heterogeneous container library for Rust.

`sovran-typemap` provides a flexible way to store different types in a single container while maintaining type-safety through runtime checks. This is particularly useful for applications that need to share state between components without requiring all components to know about all types.

## Key Features

- **Type-safe**: Values are checked at runtime to ensure type correctness
- **Thread-safe**: Built on `Arc<Mutex<_>>` for safe concurrent access
- **Ergonomic API**: Simple methods with closures for storing, retrieving, and modifying values
- **Trait Object Support**: Store and access values through trait interfaces via `TraitTypeMap`
- **Flexible**: Supports any type that implements `Any + Send + Sync` with any hashable key type
- **Comprehensive Error Handling**: Detailed error types for better debugging and recovery
- **No macros**: Pure runtime solution without complex macro magic
- **No Unsafe Code**: Relies entirely on safe Rust with no `unsafe` blocks

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
sovran-typemap = "0.4"
```

## Basic Usage

```rust
use sovran_typemap::{TypeMap, MapError};

fn main() -> Result<(), MapError> {
    // Create a new store with string keys
    let store = TypeMap::<String>::new();

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
        Err(MapError::KeyNotFound(_)) => println!("Key doesn't exist"),
        Err(MapError::TypeMismatch) => println!("Type doesn't match"),
        Err(e) => println!("Other error: {}", e),
    }

    Ok(())
}
```

## Using with_mut to Modify Values In-Place

```rust
use sovran_typemap::{TypeMap, MapError};
use std::collections::HashMap;

fn main() -> Result<(), MapError> {
    let store = TypeMap::<String>::new();

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

## TraitTypeMap: Storing Trait Objects

`TraitTypeMap` extends the concept to support trait objects. You can store values and access them either by their concrete type or through a trait interface:

```rust
use sovran_typemap::{TraitTypeMap, MapError};
use std::any::Any;

// Define a trait (must be Any + Send + Sync)
trait Animal: Any + Send + Sync {
    fn make_sound(&self) -> String;
}

#[derive(Clone)]
struct Dog {
    name: String,
    breed: String,
}

impl Dog {
    fn wag_tail(&self) -> String {
        format!("{} wags tail happily!", self.name)
    }
}

impl Animal for Dog {
    fn make_sound(&self) -> String {
        format!("{} says: Woof!", self.name)
    }
}

// Required: implement Into<Box<dyn Trait>> for your concrete type
impl Into<Box<dyn Animal>> for Dog {
    fn into(self) -> Box<dyn Animal> {
        Box::new(self)
    }
}

#[derive(Clone)]
struct Cat {
    name: String,
    lives: u8,
}

impl Cat {
    fn purr(&self) -> String {
        format!("{} purrs contentedly", self.name)
    }
}

impl Animal for Cat {
    fn make_sound(&self) -> String {
        format!("{} says: Meow!", self.name)
    }
}

impl Into<Box<dyn Animal>> for Cat {
    fn into(self) -> Box<dyn Animal> {
        Box::new(self)
    }
}

fn main() -> Result<(), MapError> {
    let store = TraitTypeMap::<String>::new();

    // Store animals with their trait type specified
    store.set_trait::<dyn Animal, _>("dog".to_string(), Dog {
        name: "Rover".to_string(),
        breed: "Golden Retriever".to_string(),
    })?;

    store.set_trait::<dyn Animal, _>("cat".to_string(), Cat {
        name: "Whiskers".to_string(),
        lives: 9,
    })?;

    // Access via concrete type - get type-specific methods
    store.with::<Dog, _, _>(&"dog".to_string(), |dog| {
        println!("Breed: {}", dog.breed);
        println!("{}", dog.wag_tail());
    })?;

    store.with::<Cat, _, _>(&"cat".to_string(), |cat| {
        println!("Lives remaining: {}", cat.lives);
        println!("{}", cat.purr());
    })?;

    // Access via trait interface - polymorphic access
    store.with_trait::<dyn Animal, _, _>(&"dog".to_string(), |animal| {
        println!("{}", animal.make_sound());
    })?;

    store.with_trait::<dyn Animal, _, _>(&"cat".to_string(), |animal| {
        println!("{}", animal.make_sound());
    })?;

    // Mutable access to concrete type
    store.with_mut::<Cat, _, _>(&"cat".to_string(), |cat| {
        cat.lives -= 1; // Uh oh, curiosity...
    })?;

    Ok(())
}
```

### When to Use Each Type

- **`TypeMap`**: When you only need to store and retrieve values by their concrete types. Simpler API, less boilerplate.

- **`TraitTypeMap`**: When you need polymorphic access to stored values through trait interfaces, or when you want to iterate over heterogeneous collections through a common trait.

## Sharing State Between Components

```rust
use sovran_typemap::{TypeMap, MapError};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

struct UserService {
    store: Arc<TypeMap<String>>,
}

struct LogService {
    store: Arc<TypeMap<String>>,
}

impl UserService {
    fn new(store: Arc<TypeMap<String>>) -> Self {
        Self { store }
    }

    fn get_user_count(&self) -> Result<usize, MapError> {
        self.store.with(&"users".to_string(), |users: &Vec<String>| {
            users.len()
        })
    }

    fn add_user(&self, username: String) -> Result<(), MapError> {
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
    fn new(store: Arc<TypeMap<String>>) -> Self {
        Self { store }
    }

    fn log(&self, message: String) -> Result<(), MapError> {
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

    fn get_recent_logs(&self, count: usize) -> Result<Vec<String>, MapError> {
        self.store.with(&"logs".to_string(), |logs: &Vec<String>| {
            logs.iter()
               .rev()
               .take(count)
               .cloned()
               .collect()
        })
    }
}

fn main() -> Result<(), MapError> {
    // Create a shared store
    let store = Arc::new(TypeMap::<String>::new());

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
use sovran_typemap::{TypeMap, MapError};

fn main() {
    let store = TypeMap::<String>::new();

    // Set a value for demonstration
    if let Err(e) = store.set("config".to_string(), vec!["setting1", "setting2"]) {
        eprintln!("Failed to store config: {}", e);
        return;
    }

    // Try to get a value with the wrong type
    match store.get::<String>(&"config".to_string()) {
        Ok(value) => println!("Config: {}", value),
        Err(MapError::KeyNotFound(_)) => println!("Config key not found"),
        Err(MapError::TypeMismatch) => println!("Config is not a String"),
        Err(MapError::LockError) => println!("Failed to acquire lock"),
    }

    // Try to access a non-existent key
    match store.get::<i32>(&"settings".to_string()) {
        Ok(value) => println!("Setting: {}", value),
        Err(MapError::KeyNotFound(_)) => println!("Settings key not found"),
        Err(e) => println!("Other error: {}", e),
    }
}
```

## Available Methods

### TypeMap

| Method | Description |
|--------|-------------|
| `new()` | Create a new empty TypeMap |
| `set(key, value)` | Store a value of any type |
| `set_with(key, closure)` | Store a value generated by a closure |
| `get<T>(key)` | Get a clone of a value |
| `with<T, F, R>(key, closure)` | Access a value with a read-only closure |
| `with_mut<T, F, R>(key, closure)` | Access a value with a read-write closure |
| `remove(key)` | Remove a value |
| `contains_key(key)` | Check if a key exists |
| `keys()` | Get all keys |
| `values<T>()` | Get all values of a specific type |
| `len()` | Get the number of items |
| `is_empty()` | Check if the store is empty |

### TraitTypeMap

| Method | Description |
|--------|-------------|
| `new()` | Create a new empty TraitTypeMap |
| `set_trait<T, U>(key, value)` | Store a value with its trait type |
| `with<T, F, R>(key, closure)` | Access a value by concrete type (read-only) |
| `with_mut<T, F, R>(key, closure)` | Access a value by concrete type (read-write) |
| `with_trait<T, F, R>(key, closure)` | Access a value through its trait interface |

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contribution

Contributions are welcome! Please feel free to submit a Pull Request.
