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
- **Multiple Container Types**: Choose the right container for your use case
- **Flexible**: Supports any type that implements `Any + Send + Sync` with any hashable key type
- **Comprehensive Error Handling**: Detailed error types for better debugging and recovery
- **No macros**: Pure runtime solution without complex macro magic
- **No Unsafe Code**: Relies entirely on safe Rust with no `unsafe` blocks

## Container Types

| Type | Key | Thread-Safe | Cloneable | Use Case |
|------|-----|-------------|-----------|----------|
| `TypeMap<K>` | Any hashable type | ✅ | ❌ | General-purpose storage with explicit keys |
| `TypeStore` | Type itself | ✅ | ❌ | Service locator / DI container (one value per type) |
| `TypeStoreValue` | Type itself | ❌ | ✅ | Cloneable state snapshots, single-threaded contexts |
| `TraitTypeMap<K>` | Any hashable type | ✅ | ❌ | Polymorphic access via trait interfaces |

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
sovran-typemap = "0.4"
```

## TypeMap: Keyed Heterogeneous Storage

`TypeMap` stores values with explicit keys, allowing multiple values of the same type under different keys.

```rust
use sovran_typemap::{TypeMap, MapError};

fn main() -> Result<(), MapError> {
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

### Modifying Values In-Place

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

    // Read current values
    let visit_count = store.with(&"counters".to_string(), |counters: &HashMap<String, i32>| {
        counters.get("visits").copied().unwrap_or(0)
    })?;

    println!("Visit count: {}", visit_count);

    Ok(())
}
```

## TypeStore: Type-Keyed Storage

`TypeStore` uses the type itself as the key, storing exactly one value per type. Perfect for dependency injection and service locator patterns.

```rust
use sovran_typemap::{TypeStore, MapError};

#[derive(Clone, Debug)]
struct DatabaseConfig {
    host: String,
    port: u16,
}

#[derive(Clone, Debug)]
struct AppConfig {
    name: String,
    debug: bool,
}

fn main() -> Result<(), MapError> {
    let store = TypeStore::new();

    // Store configurations - type IS the key
    store.set(DatabaseConfig {
        host: "localhost".to_string(),
        port: 5432,
    })?;

    store.set(AppConfig {
        name: "MyApp".to_string(),
        debug: true,
    })?;

    // Retrieve by type alone - no key needed
    let db = store.get::<DatabaseConfig>()?;
    println!("Database: {}:{}", db.host, db.port);

    // Modify in place
    store.with_mut::<AppConfig, _, _>(|cfg| {
        cfg.debug = false;
    })?;

    // Check existence by type
    if store.contains::<DatabaseConfig>()? {
        println!("Database config is registered");
    }

    // Remove by type
    store.remove::<AppConfig>()?;

    Ok(())
}
```

### Service Locator Pattern

```rust
use sovran_typemap::{TypeStore, MapError};
use std::sync::Arc;

struct Logger { prefix: String }
struct UserService { services: Arc<TypeStore> }

impl Logger {
    fn log(&self, msg: &str) {
        println!("[{}] {}", self.prefix, msg);
    }
}

impl UserService {
    fn new(services: Arc<TypeStore>) -> Self {
        Self { services }
    }

    fn create_user(&self, name: &str) -> Result<(), MapError> {
        // Access dependencies from the container
        self.services.with::<Logger, _, _>(|logger| {
            logger.log(&format!("Creating user: {}", name));
        })
    }
}

fn main() -> Result<(), MapError> {
    let services = Arc::new(TypeStore::new());
    
    // Register services
    services.set(Logger { prefix: "app".to_string() })?;
    
    // Create components with injected dependencies
    let user_service = UserService::new(Arc::clone(&services));
    user_service.create_user("alice")?;
    
    Ok(())
}
```

## TypeStoreValue: Cloneable Type-Keyed Storage

`TypeStoreValue` is like `TypeStore` but without the `Arc<Mutex<>>` wrapper, making it cloneable. Useful for state snapshots or single-threaded contexts.

```rust
use sovran_typemap::TypeStoreValue;

#[derive(Clone, Debug)]
struct GameState {
    level: u32,
    score: u64,
}

fn main() {
    let mut state = TypeStoreValue::new();

    state.set(GameState { level: 1, score: 0 });
    state.set(vec!["checkpoint1".to_string()]);

    // Take a snapshot before a dangerous operation
    let snapshot = state.clone();

    // Modify state
    state.with_mut::<GameState, _, _>(|gs| {
        gs.level = 2;
        gs.score = 1000;
    });

    // Oops, restore from snapshot
    state = snapshot;

    // Back to level 1
    assert_eq!(state.get::<GameState>().unwrap().level, 1);
}
```

Note: `TypeStoreValue` requires stored types to implement `Clone`. The API returns `Option` instead of `Result` since there's no lock that can fail.

## TraitTypeMap: Polymorphic Access

`TraitTypeMap` lets you store values and access them either by concrete type or through a trait interface:

```rust
use sovran_typemap::{TraitTypeMap, MapError};
use std::any::Any;

// Define a trait (must be Any + Send + Sync)
trait Animal: Any + Send + Sync {
    fn make_sound(&self) -> String;
}

#[derive(Clone)]
struct Dog { name: String, breed: String }

impl Dog {
    fn wag_tail(&self) -> String {
        format!("{} wags tail!", self.name)
    }
}

impl Animal for Dog {
    fn make_sound(&self) -> String {
        format!("{} says: Woof!", self.name)
    }
}

// Required: implement Into<Box<dyn Trait>> for your type
impl Into<Box<dyn Animal>> for Dog {
    fn into(self) -> Box<dyn Animal> {
        Box::new(self)
    }
}

fn main() -> Result<(), MapError> {
    let store = TraitTypeMap::<String>::new();

    store.set_trait::<dyn Animal, _>("dog".to_string(), Dog {
        name: "Rover".to_string(),
        breed: "Golden Retriever".to_string(),
    })?;

    // Access via concrete type - get type-specific methods
    store.with::<Dog, _, _>(&"dog".to_string(), |dog| {
        println!("Breed: {}", dog.breed);
        println!("{}", dog.wag_tail());
    })?;

    // Access via trait interface - polymorphic access
    store.with_trait::<dyn Animal, _, _>(&"dog".to_string(), |animal| {
        println!("{}", animal.make_sound());
    })?;

    Ok(())
}
```

## Choosing a Container

- **`TypeMap<K>`**: When you need multiple values of the same type with different keys. General-purpose heterogeneous storage.

- **`TypeStore`**: When type uniquely identifies the value and you need thread-safety. Dependency injection, configuration objects, service locators.

- **`TypeStoreValue`**: When type uniquely identifies the value but you need cloneability over thread-safety. State snapshots, undo systems, single-threaded contexts.

- **`TraitTypeMap<K>`**: When you need polymorphic access through trait interfaces, or want to iterate over values through a common trait.

## Sharing State Between Components

```rust
use sovran_typemap::{TypeMap, MapError};
use std::sync::Arc;

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

    fn add_user(&self, username: String) -> Result<(), MapError> {
        if !self.store.contains_key(&"users".to_string())? {
            self.store.set("users".to_string(), Vec::<String>::new())?;
        }
        self.store.with_mut(&"users".to_string(), |users: &mut Vec<String>| {
            users.push(username);
        })
    }
}

fn main() -> Result<(), MapError> {
    let store = Arc::new(TypeMap::<String>::new());

    let user_service = UserService::new(Arc::clone(&store));
    user_service.add_user("alice".to_string())?;
    user_service.add_user("bob".to_string())?;

    store.with(&"users".to_string(), |users: &Vec<String>| {
        println!("Users: {:?}", users);
    })?;

    Ok(())
}
```

## Error Handling

```rust
use sovran_typemap::{TypeStore, MapError};

fn main() {
    let store = TypeStore::new();

    match store.get::<String>() {
        Ok(value) => println!("Value: {}", value),
        Err(MapError::KeyNotFound(type_name)) => {
            println!("No value of type: {}", type_name)
        }
        Err(MapError::TypeMismatch) => println!("Type mismatch"),
        Err(MapError::LockError) => println!("Failed to acquire lock"),
    }
}
```

## API Reference

### TypeMap<K>

| Method | Description |
|--------|-------------|
| `new()` | Create a new empty TypeMap |
| `set(key, value)` | Store a value with a key |
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

### TypeStore

| Method | Description |
|--------|-------------|
| `new()` | Create a new empty TypeStore |
| `set(value)` | Store a value (type is the key) |
| `set_with(closure)` | Store a value generated by a closure |
| `get<T>()` | Get a clone of a value by type |
| `with<T, F, R>(closure)` | Access a value with a read-only closure |
| `with_mut<T, F, R>(closure)` | Access a value with a read-write closure |
| `remove<T>()` | Remove a value by type |
| `contains<T>()` | Check if a type exists |
| `len()` | Get the number of items |
| `is_empty()` | Check if the store is empty |

### TypeStoreValue

| Method | Description |
|--------|-------------|
| `new()` | Create a new empty TypeStoreValue |
| `set(value)` | Store a value (type is the key) |
| `set_with(closure)` | Store a value generated by a closure |
| `get<T>()` | Get a clone of a value by type (`Option`) |
| `with<T, F, R>(closure)` | Access a value with a read-only closure (`Option`) |
| `with_mut<T, F, R>(closure)` | Access a value with a read-write closure (`Option`) |
| `remove<T>()` | Remove a value by type |
| `contains<T>()` | Check if a type exists |
| `len()` | Get the number of items |
| `is_empty()` | Check if the store is empty |
| `clone()` | Clone the entire store |

### TraitTypeMap<K>

| Method | Description |
|--------|-------------|
| `new()` | Create a new empty TraitTypeMap |
| `set_trait<T, U>(key, value)` | Store a value with its trait type |
| `with<T, F, R>(key, closure)` | Access by concrete type (read-only) |
| `with_mut<T, F, R>(key, closure)` | Access by concrete type (read-write) |
| `with_trait<T, F, R>(key, closure)` | Access through trait interface |
| `remove(key)` | Remove a value |
| `contains_key(key)` | Check if a key exists |
| `keys()` | Get all keys |
| `len()` | Get the number of items |
| `is_empty()` | Check if the store is empty |

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contribution

Contributions are welcome! Please feel free to submit a Pull Request.
