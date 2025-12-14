//! # sovran-typemap
//!
//! A thread-safe, type-safe heterogeneous container library for Rust.
//!
//! `sovran-typemap` provides a flexible way to store different types in a single
//! container while maintaining type-safety through runtime checks. This is particularly
//! useful for applications that need to share state between components without requiring
//! all components to know about all types.
//!
//! ## Key Features
//!
//! - **Type-safe**: Values are checked at runtime to ensure type correctness
//! - **Thread-safe**: Most containers built on `Arc<Mutex<_>>` for safe concurrent access
//! - **Ergonomic API**: Simple methods with closures for storing, retrieving, and modifying values
//! - **Multiple Container Types**: Choose the right container for your use case
//! - **Flexible**: Supports any type that implements `Any + Send + Sync`
//! - **No macros**: Pure runtime solution without complex macro magic
//! - **No Unsafe Code**: Relies entirely on safe Rust
//!
//! ## Container Types
//!
//! | Type | Key | Thread-Safe | Cloneable | Use Case |
//! |------|-----|-------------|-----------|----------|
//! | [`TypeMap<K>`] | Any hashable type | ✅ | ❌ | General-purpose storage with explicit keys |
//! | [`TypeStore`] | Type itself | ✅ | ❌ | Service locator / DI container |
//! | [`TypeStoreValue`] | Type itself | ❌ | ✅ | Cloneable state, single-threaded contexts |
//! | [`TraitTypeMap<K>`] | Any hashable type | ✅ | ❌ | Polymorphic access via trait interfaces |
//!
//! ## Quick Examples
//!
//! ### TypeMap: Keyed Heterogeneous Storage
//!
//! ```rust
//! use sovran_typemap::{TypeMap, MapError};
//!
//! fn main() -> Result<(), MapError> {
//!     let store = TypeMap::<String>::new();
//!
//!     store.set("number".to_string(), 42i32)?;
//!     store.set("text".to_string(), "Hello!".to_string())?;
//!
//!     let num = store.get::<i32>(&"number".to_string())?;
//!     println!("Number: {}", num);
//!
//!     Ok(())
//! }
//! ```
//!
//! ### TypeStore: Type-Keyed Storage (DI Container)
//!
//! ```rust
//! use sovran_typemap::{TypeStore, MapError};
//!
//! #[derive(Clone, Debug)]
//! struct DatabaseConfig { host: String, port: u16 }
//!
//! fn main() -> Result<(), MapError> {
//!     let store = TypeStore::new();
//!
//!     // Type IS the key - no string keys needed
//!     store.set(DatabaseConfig {
//!         host: "localhost".to_string(),
//!         port: 5432,
//!     })?;
//!
//!     let config = store.get::<DatabaseConfig>()?;
//!     println!("Database: {}:{}", config.host, config.port);
//!
//!     Ok(())
//! }
//! ```
//!
//! ### TypeStoreValue: Cloneable Type-Keyed Storage
//!
//! ```rust
//! use sovran_typemap::TypeStoreValue;
//!
//! #[derive(Clone, Debug)]
//! struct GameState { level: u32, score: u64 }
//!
//! fn main() -> Result<(), ()> {
//!     let mut state = TypeStoreValue::new();
//!     state.set(GameState { level: 1, score: 0 });
//!
//!     // Take a snapshot
//!     let snapshot = state.clone();
//!
//!     // Modify original
//!     state.with_mut::<GameState, _, _>(|gs| gs.level = 2);
//!
//!     // Snapshot unchanged
//!     assert_eq!(snapshot.get::<GameState>().unwrap().level, 1);
//!     assert_eq!(state.get::<GameState>().unwrap().level, 2);
//!
//!     Ok(())
//! }
//! ```
//!
//! ### TraitTypeMap: Polymorphic Access
//!
//! ```rust
//! use sovran_typemap::{TraitTypeMap, MapError};
//! use std::any::Any;
//!
//! trait Greeter: Any + Send + Sync {
//!     fn greet(&self) -> String;
//! }
//!
//! #[derive(Clone)]
//! struct English { name: String }
//!
//! impl Greeter for English {
//!     fn greet(&self) -> String { format!("Hello, {}!", self.name) }
//! }
//!
//! impl Into<Box<dyn Greeter>> for English {
//!     fn into(self) -> Box<dyn Greeter> { Box::new(self) }
//! }
//!
//! fn main() -> Result<(), MapError> {
//!     let store = TraitTypeMap::<String>::new();
//!
//!     store.set_trait::<dyn Greeter, _>(
//!         "greeter".to_string(),
//!         English { name: "World".to_string() }
//!     )?;
//!
//!     // Access via trait interface
//!     store.with_trait::<dyn Greeter, _, _>(&"greeter".to_string(), |g| {
//!         println!("{}", g.greet());
//!     })?;
//!
//!     Ok(())
//! }
//! ```

mod any_value;
mod error;
mod map;
mod store;
mod store_value;
mod traits;

pub use error::MapError;
pub use map::TypeMap;
pub use store::TypeStore;
pub use store_value::{CloneAny, TypeStoreValue};
pub use traits::TraitTypeMap;

// Re-export std::any for convenience
pub use std::any::{Any, TypeId};
