use crate::error::MapError;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

/// A thread-safe map that stores values of a specific type
///
/// `TypeMapV` allows you to create a type-safe, thread-safe map where all values
/// must be of the same type (or implement the same trait). This is particularly
/// useful for storing collections of trait objects or other homogeneous values.
///
/// # Examples
///
/// ```
/// use sovran_typemap::{TypeMapV, MapError};
///
/// // Store trait objects
/// trait Handler: Send + Sync {
///     fn handle(&self) -> Result<(), MapError>;
/// }
///
/// let store = TypeMapV::<String, Box<dyn Handler>>::new();
///
/// // Apply an operation to all handlers
/// let result = store.apply(|key, handler| {
///     println!("Running handler {}", key);
///     handler.handle()
/// });
/// ```
#[derive(Clone, Debug)]
pub struct TypeMapV<K, V>
where
    K: Clone + Eq + Hash + Debug,
    V: Send + Sync,
{
    items: Arc<Mutex<HashMap<K, V>>>,
}

impl<K, V> TypeMapV<K, V>
where
    K: Clone + Eq + Hash + Debug,
    V: Send + Sync,
{
    /// Creates a new, empty TypeMapV
    ///
    /// # Examples
    ///
    /// ```
    /// use sovran_typemap::TypeMapV;
    ///
    /// // Create a map storing strings
    /// let string_store = TypeMapV::<String, String>::new();
    ///
    /// // Create a map storing trait objects
    /// trait MyTrait: Send + Sync {}
    /// let trait_store = TypeMapV::<u32, Box<dyn MyTrait>>::new();
    /// ```
    pub fn new() -> Self {
        Self {
            items: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Stores a value in the map
    ///
    /// # Errors
    ///
    /// Returns `MapError::LockError` if the internal lock cannot be acquired.
    pub fn set(&self, key: K, value: V) -> Result<(), MapError> {
        let mut store = self.items.lock().map_err(|_| MapError::LockError)?;
        store.insert(key, value);
        Ok(())
    }

    /// Retrieves a clone of a value from the map
    ///
    /// # Errors
    ///
    /// - Returns `MapError::LockError` if the internal lock cannot be acquired
    /// - Returns `MapError::KeyNotFound` if the key doesn't exist
    pub fn get(&self, key: &K) -> Result<V, MapError>
    where
        V: Clone,
    {
        let store = self.items.lock().map_err(|_| MapError::LockError)?;
        store
            .get(key)
            .cloned()
            .ok_or_else(|| MapError::KeyNotFound(format!("{:?}", key)))
    }

    /// Removes a value from the map
    ///
    /// # Errors
    ///
    /// Returns `MapError::LockError` if the internal lock cannot be acquired.
    ///
    /// # Returns
    ///
    /// Returns `Ok(true)` if the key was present and removed, `Ok(false)` if not present.
    pub fn remove(&self, key: &K) -> Result<bool, MapError> {
        let mut store = self.items.lock().map_err(|_| MapError::LockError)?;
        Ok(store.remove(key).is_some())
    }

    /// Applies a function to all key-value pairs in the map
    ///
    /// This method allows you to perform operations on all stored values while
    /// maintaining thread safety. The function is called with a reference to both
    /// the key and value for each entry.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sovran_typemap::{TypeMapV, MapError};
    /// trait Handler: Send + Sync {
    ///     fn handle(&self) -> Result<(), MapError>;
    /// }
    ///
    /// let store = TypeMapV::<String, Box<dyn Handler>>::new();
    ///
    /// // Apply to all handlers
    /// let result = store.apply(|key, handler| {
    ///     handler.handle().map_err(|_| MapError::LockError)
    /// });
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `MapError::LockError` if the internal lock cannot be acquired,
    /// or any error returned by the provided function.
    pub fn apply<F>(&self, mut f: F) -> Result<(), MapError>
    where
        F: FnMut(&K, &V) -> Result<(), MapError>,
    {
        let store = self.items.lock().map_err(|_| MapError::LockError)?;
        for (key, value) in store.iter() {
            f(key, value)?;
        }
        Ok(())
    }

    /// Returns the number of entries in the map
    ///
    /// # Errors
    ///
    /// Returns `MapError::LockError` if the internal lock cannot be acquired.
    pub fn len(&self) -> Result<usize, MapError> {
        let store = self.items.lock().map_err(|_| MapError::LockError)?;
        Ok(store.len())
    }

    /// Returns true if the map contains no entries
    ///
    /// # Errors
    ///
    /// Returns `MapError::LockError` if the internal lock cannot be acquired.
    pub fn is_empty(&self) -> Result<bool, MapError> {
        let store = self.items.lock().map_err(|_| MapError::LockError)?;
        Ok(store.is_empty())
    }

    /// Returns true if the map contains the specified key
    ///
    /// # Errors
    ///
    /// Returns `MapError::LockError` if the internal lock cannot be acquired.
    pub fn contains_key(&self, key: &K) -> Result<bool, MapError> {
        let store = self.items.lock().map_err(|_| MapError::LockError)?;
        Ok(store.contains_key(key))
    }

    /// Returns a vector of all keys in the map
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

    /// Returns a vector of all values in the map
    ///
    /// # Errors
    ///
    /// Returns `MapError::LockError` if the internal lock cannot be acquired.
    pub fn values(&self) -> Result<Vec<V>, MapError>
    where
        V: Clone,
    {
        let store = self.items.lock().map_err(|_| MapError::LockError)?;
        Ok(store.values().cloned().collect())
    }

    /// Gets a value by executing a closure with read access
    ///
    /// This method allows you to perform operations on a stored value without
    /// requiring the value to implement Clone. It's useful for inspecting values
    /// or computing derived values.
    ///
    /// # Arguments
    ///
    /// * `key` - The key for the value you want to access
    /// * `f` - A closure that takes a reference to the value and returns a result
    ///
    /// # Examples
    ///
    /// ```
    /// use sovran_typemap::{TypeMapV, MapError};
    ///
    /// let store = TypeMapV::<String, Vec<i32>>::new();
    /// store.set("numbers".to_string(), vec![1, 2, 3])?;
    ///
    /// // Get the length without cloning the vector
    /// let length = store.with(&"numbers".to_string(), |v| v.len())?;
    /// assert_eq!(length, 3);
    ///
    /// // Check if vector contains a value
    /// let has_two = store.with(&"numbers".to_string(), |v| v.contains(&2))?;
    /// assert!(has_two);
    /// # Ok::<(), MapError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `MapError::LockError` if the internal lock cannot be acquired
    /// Returns `MapError::KeyNotFound` if the key doesn't exist
    pub fn with<F, R>(&self, key: &K, f: F) -> Result<R, MapError>
    where
        F: FnOnce(&V) -> R,
    {
        let store = self.items.lock().map_err(|_| MapError::LockError)?;
        let value = store.get(key)
            .ok_or_else(|| MapError::KeyNotFound(format!("{:?}", key)))?;
        Ok(f(value))
    }

    /// Gets a value by executing a closure with write access
    ///
    /// This method allows you to modify a stored value in place without
    /// replacing it entirely. It's useful for updating collections or
    /// complex structures.
    ///
    /// # Arguments
    ///
    /// * `key` - The key for the value you want to modify
    /// * `f` - A closure that takes a mutable reference to the value and returns a result
    ///
    /// # Examples
    ///
    /// ```
    /// use sovran_typemap::{TypeMapV, MapError};
    ///
    /// let store = TypeMapV::<String, Vec<i32>>::new();
    /// store.set("numbers".to_string(), vec![1, 2, 3])?;
    ///
    /// // Modify the vector in place
    /// let new_len = store.with_mut(&"numbers".to_string(), |v| {
    ///     v.push(4);
    ///     v.push(5);
    ///     v.len()
    /// })?;
    ///
    /// assert_eq!(new_len, 5);
    /// assert_eq!(store.get(&"numbers".to_string())?, vec![1, 2, 3, 4, 5]);
    /// # Ok::<(), MapError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `MapError::LockError` if the internal lock cannot be acquired
    /// Returns `MapError::KeyNotFound` if the key doesn't exist
    pub fn with_mut<F, R>(&self, key: &K, f: F) -> Result<R, MapError>
    where
        F: FnOnce(&mut V) -> R,
    {
        let mut store = self.items.lock().map_err(|_| MapError::LockError)?;
        let value = store.get_mut(key)
            .ok_or_else(|| MapError::KeyNotFound(format!("{:?}", key)))?;
        Ok(f(value))
    }
}

impl<K, V> Default for TypeMapV<K, V>
where
    K: Clone + Eq + Hash + Debug,
    V: Send + Sync,
{
    fn default() -> Self {
        Self::new()
    }
}
