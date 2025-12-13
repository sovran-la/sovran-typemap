// src/traits.rs
use crate::MapError;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

pub(crate) struct TypeMapValue {
    concrete_type_id: TypeId,
    trait_type_id: TypeId,
    concrete_value: Box<dyn Any + Send + Sync>,
    trait_object: Box<dyn Any + Send + Sync>,
}

/// A thread-safe heterogeneous container that supports trait object access.
///
/// `TraitTypeMap` extends the concept of `TypeMap` to support storing values
/// that can be accessed either by their concrete type or through a trait interface.
/// This is useful when you need polymorphic access to stored values.
///
/// # Examples
///
/// ```
/// use sovran_typemap::{TraitTypeMap, MapError};
/// use std::any::Any;
///
/// // Define a trait
/// trait Greeter: Any + Send + Sync {
///     fn greet(&self) -> String;
/// }
///
/// #[derive(Clone)]
/// struct EnglishGreeter { name: String }
///
/// impl Greeter for EnglishGreeter {
///     fn greet(&self) -> String { format!("Hello, {}!", self.name) }
/// }
///
/// impl Into<Box<dyn Greeter>> for EnglishGreeter {
///     fn into(self) -> Box<dyn Greeter> { Box::new(self) }
/// }
///
/// let store = TraitTypeMap::<String>::new();
/// store.set_trait::<dyn Greeter, _>("greeter".to_string(), EnglishGreeter { name: "World".to_string() }).unwrap();
///
/// // Access via trait
/// store.with_trait::<dyn Greeter, _, _>(&"greeter".to_string(), |g| {
///     assert_eq!(g.greet(), "Hello, World!");
/// }).unwrap();
/// ```
pub struct TraitTypeMap<K> {
    items: Arc<Mutex<HashMap<K, TypeMapValue>>>,
}

impl<K> TraitTypeMap<K>
where
    K: Clone + Eq + Hash + Debug,
{
    /// Creates a new, empty TraitTypeMap.
    pub fn new() -> Self {
        Self {
            items: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Stores a value with its associated trait type.
    ///
    /// The value can later be accessed either by its concrete type or through
    /// the trait interface.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The trait type (e.g., `dyn MyTrait`)
    /// * `U` - The concrete type that implements the trait
    ///
    /// # Errors
    ///
    /// Returns `MapError::LockError` if the internal lock cannot be acquired.
    pub fn set_trait<T, U>(&self, key: K, value: U) -> Result<(), MapError>
    where
        T: ?Sized + Any + Send + Sync + 'static,
        U: 'static + Into<Box<T>> + Send + Sync + Clone,
    {
        let type_map_value = TypeMapValue {
            concrete_type_id: TypeId::of::<U>(),
            trait_type_id: TypeId::of::<T>(),
            concrete_value: Box::new(value.clone()),
            trait_object: Box::new(value.into()),
        };

        let mut store = self.items.lock().map_err(|_| MapError::LockError)?;
        store.insert(key, type_map_value);
        Ok(())
    }

    /// Accesses a value by its concrete type with a read-only closure.
    ///
    /// # Errors
    ///
    /// - Returns `MapError::LockError` if the internal lock cannot be acquired
    /// - Returns `MapError::KeyNotFound` if the key doesn't exist
    /// - Returns `MapError::TypeMismatch` if the concrete type doesn't match
    pub fn with<V: 'static, F, R>(&self, key: &K, f: F) -> Result<R, MapError>
    where
        F: FnOnce(&V) -> R,
    {
        let guard = self.items.lock().map_err(|_| MapError::LockError)?;
        let value = guard
            .get(key)
            .ok_or_else(|| MapError::KeyNotFound(format!("{:?}", key)))?;

        if value.concrete_type_id == TypeId::of::<V>() {
            if let Some(concrete) = value.concrete_value.downcast_ref::<V>() {
                return Ok(f(concrete));
            }
        }

        Err(MapError::TypeMismatch)
    }

    /// Accesses a value by its concrete type with a read-write closure.
    ///
    /// # Errors
    ///
    /// - Returns `MapError::LockError` if the internal lock cannot be acquired
    /// - Returns `MapError::KeyNotFound` if the key doesn't exist
    /// - Returns `MapError::TypeMismatch` if the concrete type doesn't match
    pub fn with_mut<V: 'static, F, R>(&self, key: &K, f: F) -> Result<R, MapError>
    where
        F: FnOnce(&mut V) -> R,
    {
        let mut guard = self.items.lock().map_err(|_| MapError::LockError)?;
        let value = guard
            .get_mut(key)
            .ok_or_else(|| MapError::KeyNotFound(format!("{:?}", key)))?;

        if value.concrete_type_id == TypeId::of::<V>() {
            if let Some(concrete) = value.concrete_value.downcast_mut::<V>() {
                return Ok(f(concrete));
            }
        }

        Err(MapError::TypeMismatch)
    }

    /// Accesses a value through its trait interface with a read-only closure.
    ///
    /// This enables polymorphic access to stored values without knowing
    /// their concrete type.
    ///
    /// # Errors
    ///
    /// - Returns `MapError::LockError` if the internal lock cannot be acquired
    /// - Returns `MapError::KeyNotFound` if the key doesn't exist
    /// - Returns `MapError::TypeMismatch` if the trait type doesn't match
    pub fn with_trait<T, F, R>(&self, key: &K, f: F) -> Result<R, MapError>
    where
        T: ?Sized + Any + Send + Sync + 'static,
        F: FnOnce(&T) -> R,
    {
        let guard = self.items.lock().map_err(|_| MapError::LockError)?;
        let value = guard
            .get(key)
            .ok_or_else(|| MapError::KeyNotFound(format!("{:?}", key)))?;

        if value.trait_type_id == TypeId::of::<T>() {
            if let Some(boxed_trait) = value.trait_object.downcast_ref::<Box<T>>() {
                return Ok(f(&**boxed_trait));
            }
        }

        Err(MapError::TypeMismatch)
    }

    /// Removes a value from the store.
    ///
    /// # Errors
    ///
    /// Returns `MapError::LockError` if the internal lock cannot be acquired.
    ///
    /// # Returns
    ///
    /// Returns `Ok(true)` if the key was present and removed, `Ok(false)` otherwise.
    pub fn remove(&self, key: &K) -> Result<bool, MapError> {
        let mut store = self.items.lock().map_err(|_| MapError::LockError)?;
        Ok(store.remove(key).is_some())
    }

    /// Checks if a key exists in the store.
    ///
    /// # Errors
    ///
    /// Returns `MapError::LockError` if the internal lock cannot be acquired.
    pub fn contains_key(&self, key: &K) -> Result<bool, MapError> {
        let store = self.items.lock().map_err(|_| MapError::LockError)?;
        Ok(store.contains_key(key))
    }

    /// Gets all keys in the store.
    ///
    /// # Errors
    ///
    /// Returns `MapError::LockError` if the internal lock cannot be acquired.
    pub fn keys(&self) -> Result<Vec<K>, MapError>
    where
        K: Clone,
    {
        let store = self.items.lock().map_err(|_| MapError::LockError)?;
        Ok(store.keys().cloned().collect())
    }

    /// Gets the number of items in the store.
    ///
    /// # Errors
    ///
    /// Returns `MapError::LockError` if the internal lock cannot be acquired.
    pub fn len(&self) -> Result<usize, MapError> {
        let store = self.items.lock().map_err(|_| MapError::LockError)?;
        Ok(store.len())
    }

    /// Checks if the store is empty.
    ///
    /// # Errors
    ///
    /// Returns `MapError::LockError` if the internal lock cannot be acquired.
    pub fn is_empty(&self) -> Result<bool, MapError> {
        let store = self.items.lock().map_err(|_| MapError::LockError)?;
        Ok(store.is_empty())
    }
}

impl<K> Default for TraitTypeMap<K>
where
    K: Clone + Eq + Hash + Debug,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::TypeId;

    // Define a test trait
    trait Animal: Any + Send + Sync {
        fn make_sound(&self) -> String;
    }

    // And some implementations
    #[derive(Debug, Clone)]
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

    impl Into<Box<dyn Animal>> for Dog {
        fn into(self) -> Box<dyn Animal> {
            Box::new(self)
        }
    }

    #[derive(Debug, Clone)]
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

    #[test]
    fn test_single_type() -> Result<(), MapError> {
        println!("\nStarting test_single_type");
        let store = TraitTypeMap::<String>::new();

        let dog = Dog {
            name: "Rover".to_string(),
            breed: "Golden Retriever".to_string(),
        };

        println!("Dog TypeId: {:?}", TypeId::of::<Dog>());
        println!("Animal TypeId: {:?}", TypeId::of::<dyn Animal>());
        println!("TypeMapValue TypeId: {:?}", TypeId::of::<TypeMapValue>());

        // Store the dog
        store.set_trait::<dyn Animal, _>("dog".to_string(), dog)?;

        // Try to access it
        store.with::<Dog, _, _>(&"dog".to_string(), |dog| {
            assert_eq!(dog.breed, "Golden Retriever");
            Ok::<(), MapError>(())
        })?
    }

    #[test]
    fn test_trait_storage_and_access() -> Result<(), MapError> {
        let store = TraitTypeMap::<String>::new();

        // Store different animals
        store.set_trait::<dyn Animal, _>(
            "dog".to_string(),
            Dog {
                name: "Rover".to_string(),
                breed: "Golden Retriever".to_string(),
            },
        )?;

        store.set_trait::<dyn Animal, _>(
            "cat".to_string(),
            Cat {
                name: "Whiskers".to_string(),
                lives: 9,
            },
        )?;

        // Access via concrete type
        store.with::<Dog, _, _>(&"dog".to_string(), |dog| {
            assert_eq!(dog.breed, "Golden Retriever");
            assert_eq!(dog.wag_tail(), "Rover wags tail happily!");
        })?;

        store.with::<Cat, _, _>(&"cat".to_string(), |cat| {
            assert_eq!(cat.lives, 9);
            assert_eq!(cat.purr(), "Whiskers purrs contentedly");
        })?;

        // Verify type safety
        assert!(store.with::<Cat, _, _>(&"dog".to_string(), |_| {}).is_err());
        assert!(store.with::<Dog, _, _>(&"cat".to_string(), |_| {}).is_err());

        Ok(())
    }

    #[test]
    fn test_mutable_access() -> Result<(), MapError> {
        let store = TraitTypeMap::<String>::new();

        // Store a cat
        store.set_trait::<dyn Animal, _>(
            "cat".to_string(),
            Cat {
                name: "Whiskers".to_string(),
                lives: 9,
            },
        )?;

        // Modify via concrete type
        store.with_mut::<Cat, _, _>(&"cat".to_string(), |cat| {
            cat.lives -= 1;
        })?;

        // Verify modification
        store.with::<Cat, _, _>(&"cat".to_string(), |cat| {
            assert_eq!(cat.lives, 8);
        })?;

        Ok(())
    }

    #[test]
    fn test_type_errors() {
        let store = TraitTypeMap::<String>::new();

        // Store a dog
        store
            .set_trait::<dyn Animal, _>(
                "pet".to_string(),
                Dog {
                    name: "Rover".to_string(),
                    breed: "Golden Retriever".to_string(),
                },
            )
            .unwrap();

        // Try to access as wrong type
        match store.with::<Cat, _, _>(&"pet".to_string(), |_| {}) {
            Err(MapError::TypeMismatch) => (), // Expected
            _ => panic!("Should have gotten type mismatch error"),
        }

        // Try to access non-existent key
        match store.with::<Dog, _, _>(&"nonexistent".to_string(), |_| {}) {
            Err(MapError::KeyNotFound(_)) => (), // Expected
            _ => panic!("Should have gotten key not found error"),
        }
    }

    #[test]
    fn test_trait_access() -> Result<(), MapError> {
        println!("\nStarting test_trait_access");
        let store = TraitTypeMap::<String>::new();

        let dog = Dog {
            name: "Rover".to_string(),
            breed: "Golden Retriever".to_string(),
        };

        // Store the dog
        store.set_trait::<dyn Animal, _>("dog".to_string(), dog)?;

        // Try to access it as dyn Animal
        store.with_trait::<dyn Animal, _, _>(&"dog".to_string(), |animal| {
            assert_eq!(animal.make_sound(), "Rover says: Woof!");
            Ok(())
        })?
    }

    #[test]
    fn test_remove() -> Result<(), MapError> {
        let store = TraitTypeMap::<String>::new();

        store.set_trait::<dyn Animal, _>(
            "dog".to_string(),
            Dog {
                name: "Rover".to_string(),
                breed: "Golden Retriever".to_string(),
            },
        )?;

        assert!(store.contains_key(&"dog".to_string())?);
        assert!(store.remove(&"dog".to_string())?);
        assert!(!store.contains_key(&"dog".to_string())?);
        assert!(!store.remove(&"dog".to_string())?); // Already removed

        Ok(())
    }

    #[test]
    fn test_keys_len_is_empty() -> Result<(), MapError> {
        let store = TraitTypeMap::<String>::new();

        assert!(store.is_empty()?);
        assert_eq!(store.len()?, 0);
        assert!(store.keys()?.is_empty());

        store.set_trait::<dyn Animal, _>(
            "dog".to_string(),
            Dog {
                name: "Rover".to_string(),
                breed: "Golden Retriever".to_string(),
            },
        )?;

        store.set_trait::<dyn Animal, _>(
            "cat".to_string(),
            Cat {
                name: "Whiskers".to_string(),
                lives: 9,
            },
        )?;

        assert!(!store.is_empty()?);
        assert_eq!(store.len()?, 2);

        let mut keys = store.keys()?;
        keys.sort();
        assert_eq!(keys, vec!["cat".to_string(), "dog".to_string()]);

        Ok(())
    }
}
