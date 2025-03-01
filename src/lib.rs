//! # sovran-typemap
//!
//! A thread-safe, type-safe heterogeneous container library.
//!
//! `sovran-typemap` provides a flexible way to store different types in a single
//! container while maintaining type-safety through runtime checks. This is particularly
//! useful for applications that need to share state between components without requiring
//! all components to know about all types.
//!
//! ## Key Features
//!
//! - **Type-safe**: Values are checked at runtime to ensure type correctness
//! - **Thread-safe**: Built on `Arc<Mutex<_>>` for safe concurrent access
//! - **Ergonomic API**: Simple methods for storing, retrieving, and modifying values
//! - **Flexible**: Supports any type that implements `Any + Send + Sync`
//! - **No macros**: Pure runtime solution without complex macro magic
//!
//! ## Usage Examples
//!
//! ### Basic Usage
//!
//! ```rust
//! use sovran_typemap::{TypeMap, MapError};
//!
//! fn main() -> Result<(), MapError> {
//!     // Create a new store with string keys
//!     let store = TypeMap::<String>::new();
//!
//!     // Store values of different types
//!     store.set("number".to_string(), 42i32)?;
//!     store.set("text".to_string(), "Hello, world!".to_string())?;
//!     store.set("data".to_string(), vec![1, 2, 3, 4, 5])?;
//!
//!     // Retrieve values in a type-safe way
//!     let num = store.get::<i32>(&"number".to_string())?;
//!     let text = store.get::<String>(&"text".to_string())?;
//!
//!     println!("Number: {}", num);
//!     println!("Text: {}", text);
//!
//!     // Handle errors properly
//!     match store.get::<bool>(&"nonexistent".to_string()) {
//!         Ok(value) => println!("Value: {}", value),
//!         Err(MapError::KeyNotFound(key)) => println!("Key ({}) doesn't exist", key),
//!         Err(MapError::TypeMismatch) => println!("Type doesn't match"),
//!         Err(e) => println!("Other error: {}", e),
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Using with_mut to Modify Values In-Place
//!
//! ```rust
//! use sovran_typemap::{TypeMap, MapError};
//! use std::collections::HashMap;
//!
//! fn main() -> Result<(), MapError> {
//!     let store = TypeMap::<String>::new();
//!
//!     // Initialize a counter map
//!     let mut counters = HashMap::new();
//!     counters.insert("visits".to_string(), 0);
//!     store.set("counters".to_string(), counters)?;
//!
//!     // Update a counter in-place
//!     store.with_mut(&"counters".to_string(), |counters: &mut HashMap<String, i32>| {
//!         let visits = counters.entry("visits".to_string()).or_insert(0);
//!         *visits += 1;
//!     })?;
//!
//!     // Add a new counter
//!     store.with_mut(&"counters".to_string(), |counters: &mut HashMap<String, i32>| {
//!         counters.insert("api_calls".to_string(), 1);
//!     })?;
//!
//!     // Read current values
//!     let visit_count = store.with(&"counters".to_string(), |counters: &HashMap<String, i32>| {
//!         counters.get("visits").copied().unwrap_or(0)
//!     })?;
//!
//!     println!("Visit count: {}", visit_count);
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Sharing State Between Components
//!
//! ```rust,no_run
//! use sovran_typemap::{TypeMap, MapError};
//! use std::sync::Arc;
//! use std::time::{SystemTime, UNIX_EPOCH};
//!
//! struct UserService {
//!     store: Arc<TypeMap<String>>,
//! }
//!
//! struct LogService {
//!     store: Arc<TypeMap<String>>,
//! }
//!
//! impl UserService {
//!     fn new(store: Arc<TypeMap<String>>) -> Self {
//!         Self { store }
//!     }
//!
//!     fn get_user_count(&self) -> Result<usize, MapError> {
//!         self.store.with(&"users".to_string(), |users: &Vec<String>| {
//!             users.len()
//!         })
//!     }
//!
//!     fn add_user(&self, username: String) -> Result<(), MapError> {
//!         // Initialize users vector if it doesn't exist yet
//!         if !self.store.contains_key(&"users".to_string())? {
//!             self.store.set("users".to_string(), Vec::<String>::new())?;
//!         }
//!
//!         // Add a user
//!         self.store.with_mut(&"users".to_string(), |users: &mut Vec<String>| {
//!             users.push(username);
//!         })
//!     }
//! }
//!
//! impl LogService {
//!     fn new(store: Arc<TypeMap<String>>) -> Self {
//!         Self { store }
//!     }
//!
//!     fn log(&self, message: String) -> Result<(), MapError> {
//!         // Initialize logs if they don't exist
//!         if !self.store.contains_key(&"logs".to_string())? {
//!             self.store.set("logs".to_string(), Vec::<String>::new())?;
//!         }
//!
//!         // Add log entry with timestamp using standard library
//!         self.store.with_mut(&"logs".to_string(), |logs: &mut Vec<String>| {
//!             let now = SystemTime::now()
//!                 .duration_since(UNIX_EPOCH)
//!                 .unwrap_or_default()
//!                 .as_secs();
//!             logs.push(format!("[{}] {}", now, message));
//!         })
//!     }
//!
//!     fn get_recent_logs(&self, count: usize) -> Result<Vec<String>, MapError> {
//!         self.store.with(&"logs".to_string(), |logs: &Vec<String>| {
//!             logs.iter()
//!                .rev()
//!                .take(count)
//!                .cloned()
//!                .collect()
//!         })
//!     }
//! }
//!
//! fn main() -> Result<(), MapError> {
//!     // Create a shared store
//!     let store = Arc::new(TypeMap::<String>::new());
//!
//!     // Create services that share the store
//!     let user_service = UserService::new(Arc::clone(&store));
//!     let log_service = LogService::new(Arc::clone(&store));
//!
//!     // Use the services
//!     user_service.add_user("alice".to_string())?;
//!     log_service.log("User alice added".to_string())?;
//!
//!     user_service.add_user("bob".to_string())?;
//!     log_service.log("User bob added".to_string())?;
//!
//!     // Get information from both services
//!     println!("User count: {}", user_service.get_user_count()?);
//!     
//!     println!("Recent logs:");
//!     for log in log_service.get_recent_logs(5)? {
//!         println!("  {}", log);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Error Handling
//!
//! ```rust
//! use sovran_typemap::{TypeMap, MapError};
//!
//! let store = TypeMap::<String>::new();
//!
//! // Set a value for demonstration
//! if let Err(e) = store.set("config".to_string(), vec!["setting1", "setting2"]) {
//!     eprintln!("Failed to store config: {}", e);
//!     return;
//! }
//!
//! // Try to get a value with the wrong type
//! match store.get::<String>(&"config".to_string()) {
//! Ok(value) => println!("Config: {}", value),
//!     Err(MapError::KeyNotFound(_)) => println!("Config key not found"),
//!     Err(MapError::TypeMismatch) => println!("Config is not a String"),
//!     Err(MapError::LockError) => println!("Failed to acquire lock"),
//! }
//!
//! // Try to access a non-existent key
//! match store.get::<i32>(&"settings".to_string()) {
//!     Ok(value) => println!("Setting: {}", value),
//!     Err(MapError::KeyNotFound(_)) => println!("Settings key not found"),
//!     Err(e) => println!("Other error: {}", e),
//! }
//! ```

mod any_value;
mod error;
mod map;
mod typed;

pub use error::MapError;
pub use map::TypeMap;
pub use typed::TypeMapV;

// Re-export std::any for convenience
pub use std::any::{Any, TypeId};
