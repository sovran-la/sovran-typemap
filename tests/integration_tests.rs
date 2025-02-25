use sovran_typemap::{StoreError, TypeStore};
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;

#[test]
fn test_basic_operations() {
    let store: TypeStore<String> = TypeStore::new();

    // Store a value
    store.set("key".to_string(), 42i32).unwrap();

    // Check if key exists
    assert!(store.contains_key(&"key".to_string()).unwrap());

    // Get the value
    let value = store
        .with(&"key".to_string(), |val: &i32| {
            assert_eq!(*val, 42);
            *val
        })
        .unwrap();
    assert_eq!(value, 42);

    // Update the value
    store
        .with_mut(&"key".to_string(), |val: &mut i32| {
            *val = 100;
        })
        .unwrap();

    let get_value = store.get::<i32>(&"key".to_string()).unwrap();
    assert_eq!(get_value, 100);

    // Check the updated value
    let value = store.with(&"key".to_string(), |val: &i32| *val).unwrap();
    assert_eq!(value, 100);

    // Replace with entirely new value of different type
    store
        .set("key".to_string(), "new value".to_string())
        .unwrap();

    // Access with new type
    let string_value = store
        .with(&"key".to_string(), |val: &String| val.clone())
        .unwrap();
    assert_eq!(string_value, "new value");

    // Remove the value
    assert!(store.remove(&"key".to_string()).unwrap());

    // Check that it's gone
    assert!(!store.contains_key(&"key".to_string()).unwrap());
}

#[test]
fn test_type_safety() {
    let store: TypeStore<String> = TypeStore::new();

    // Store a string
    store.set("key".to_string(), "hello".to_string()).unwrap();

    // Try to get it as the wrong type
    let result = store.with(&"key".to_string(), |val: &i32| *val);
    assert!(matches!(result, Err(StoreError::TypeMismatch)));

    // Get it as the correct type
    let value = store
        .with(&"key".to_string(), |val: &String| val.clone())
        .unwrap();
    assert_eq!(value, "hello");
}

#[test]
fn test_multiple_types() {
    let store: TypeStore<String> = TypeStore::new();

    // Store different types
    store.set("int".to_string(), 42i32).unwrap();
    store
        .set("string".to_string(), "hello".to_string())
        .unwrap();
    store.set("float".to_string(), 3.14f64).unwrap();

    // Get them back
    let int_val = store.with(&"int".to_string(), |val: &i32| *val).unwrap();
    assert_eq!(int_val, 42);

    let string_val = store
        .with(&"string".to_string(), |val: &String| val.clone())
        .unwrap();
    assert_eq!(string_val, "hello");

    let float_val = store.with(&"float".to_string(), |val: &f64| *val).unwrap();
    assert_eq!(float_val, 3.14);

    // Get the keys
    let keys = store.keys().unwrap();
    assert_eq!(keys.len(), 3);
    assert!(keys.contains(&"int".to_string()));
    assert!(keys.contains(&"string".to_string()));
    assert!(keys.contains(&"float".to_string()));
}

#[test]
fn test_thread_safety() {
    let store = Arc::new(TypeStore::<String>::new());

    // Store a value
    store.set("counter".to_string(), 0i32).unwrap();

    // Create multiple threads to increment the counter
    let mut handles = vec![];
    for _ in 0..10 {
        let store_clone = Arc::clone(&store);
        handles.push(thread::spawn(move || {
            for _ in 0..100 {
                store_clone
                    .with_mut(&"counter".to_string(), |counter: &mut i32| {
                        *counter += 1;
                    })
                    .unwrap();
            }
        }));
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Check the final value
    let final_value = store
        .with(&"counter".to_string(), |val: &i32| *val)
        .unwrap();
    assert_eq!(final_value, 1000); // 10 threads * 100 increments
}

#[test]
fn test_error_handling() {
    let store: TypeStore<String> = TypeStore::new();

    // Try to get a non-existent key
    let result = store.with(&"nonexistent".to_string(), |val: &i32| *val);
    assert!(matches!(result, Err(StoreError::KeyNotFound)));

    // Store a value and try to get it with the wrong type
    store.set("key".to_string(), 42i32).unwrap();
    let result = store.with(&"key".to_string(), |val: &String| val.clone());
    assert!(matches!(result, Err(StoreError::TypeMismatch)));

    // Try to modify a non-existent key
    let result = store.with_mut(&"nonexistent".to_string(), |val: &mut i32| {
        *val = 100;
    });
    assert!(matches!(result, Err(StoreError::KeyNotFound)));

    // Try to remove a non-existent key
    let removed = store.remove(&"nonexistent".to_string()).unwrap();
    assert!(!removed);
}

#[test]
fn test_empty_store_operations() {
    let store: TypeStore<String> = TypeStore::new();

    // Check that the store is empty
    assert!(store.is_empty().unwrap());
    assert_eq!(store.len().unwrap(), 0);

    // Check that keys returns an empty vector
    let keys = store.keys().unwrap();
    assert!(keys.is_empty());
}

#[test]
fn test_custom_key_types() {
    // Test with integer keys
    let store: TypeStore<i32> = TypeStore::new();

    store.set(1, "one".to_string()).unwrap();
    store.set(2, "two".to_string()).unwrap();

    let value = store.with(&1, |val: &String| val.clone()).unwrap();
    assert_eq!(value, "one");

    // Test with tuple keys
    let store: TypeStore<(i32, i32)> = TypeStore::new();

    store.set((1, 2), "point".to_string()).unwrap();

    let value = store.with(&(1, 2), |val: &String| val.clone()).unwrap();
    assert_eq!(value, "point");
}

#[test]
fn test_error_display() {
    // Test error.rs Display implementation
    let lock_error = StoreError::LockError;
    let key_not_found = StoreError::KeyNotFound;
    let type_mismatch = StoreError::TypeMismatch;

    assert_eq!(format!("{}", lock_error), "Failed to acquire lock");
    assert_eq!(format!("{}", key_not_found), "Key not found in store");
    assert_eq!(
        format!("{}", type_mismatch),
        "Type mismatch for the requested key"
    );

    // Test Debug implementation
    assert!(format!("{:?}", lock_error).contains("LockError"));
}

#[test]
fn test_set_with() {
    let store: TypeStore<String> = TypeStore::new();

    // Test set_with functionality
    let result = store.set_with("expensive".to_string(), || {
        // Simulate an expensive operation
        let mut data = Vec::new();
        for i in 0..10 {
            data.push(i);
        }
        data
    });

    assert!(result.is_ok());

    // Verify the data was stored correctly
    let data = store.get::<Vec<i32>>(&"expensive".to_string()).unwrap();
    assert_eq!(data, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

    // Test with a different type
    store
        .set_with("config".to_string(), || {
            let mut map = HashMap::new();
            map.insert("key".to_string(), "value".to_string());
            map
        })
        .unwrap();

    let config = store
        .with(&"config".to_string(), |c: &HashMap<String, String>| {
            c.get("key").cloned()
        })
        .unwrap();

    assert_eq!(config, Some("value".to_string()));
}

#[test]
fn test_with_mut_type_mismatch() {
    let store: TypeStore<String> = TypeStore::new();

    // Store a string
    store.set("key".to_string(), "value".to_string()).unwrap();

    // Try to access it as an integer (should produce TypeMismatch)
    let result = store.with_mut(&"key".to_string(), |_: &mut i32| {
        // This shouldn't execute
        panic!("Should not reach here");
    });

    assert!(matches!(result, Err(StoreError::TypeMismatch)));
}

#[test]
fn test_default_implementation() {
    // Test the Default implementation
    let store: TypeStore<String> = Default::default();

    // Verify it works like a new store
    assert!(store.is_empty().unwrap());

    // Store something to verify functionality
    store.set("test".to_string(), 42).unwrap();
    assert_eq!(store.get::<i32>(&"test".to_string()).unwrap(), 42);
}
