use sovran_typemap::{MapError, TypeMapV};

// Test trait for checking trait object storage
trait TestHandler: Send + Sync {
    fn handle(&self) -> Result<String, MapError>;
}

struct SimpleHandler(String);
impl TestHandler for SimpleHandler {
    fn handle(&self) -> Result<String, MapError> {
        Ok(self.0.clone())
    }
}

#[test]
fn test_basic_operations() -> Result<(), MapError> {
    let store = TypeMapV::<String, i32>::new();

    // Test empty state
    assert!(store.is_empty()?);
    assert_eq!(store.len()?, 0);

    // Test insertion and retrieval
    store.set("test".to_string(), 42)?;
    assert_eq!(store.get(&"test".to_string())?, 42);

    // Test contains_key
    assert!(store.contains_key(&"test".to_string())?);
    assert!(!store.contains_key(&"nope".to_string())?);

    // Test removal
    assert!(store.remove(&"test".to_string())?);
    assert!(!store.remove(&"test".to_string())?);
    assert!(store.is_empty()?);

    Ok(())
}

#[test]
fn test_keys_and_values() -> Result<(), MapError> {
    let store = TypeMapV::<String, i32>::new();

    store.set("one".to_string(), 1)?;
    store.set("two".to_string(), 2)?;
    store.set("three".to_string(), 3)?;

    let mut keys = store.keys()?;
    let mut values = store.values()?;

    // Sort both for stable testing
    keys.sort();
    values.sort();

    let mut expected_keys = vec!["one", "three", "two"];
    expected_keys.sort();

    assert_eq!(keys, expected_keys);
    assert_eq!(values, vec![1, 2, 3]);

    Ok(())
}

#[test]
fn test_trait_objects() -> Result<(), MapError> {
    let store = TypeMapV::<String, Box<dyn TestHandler>>::new();

    store.set(
        "first".to_string(),
        Box::new(SimpleHandler("Hello".to_string())),
    )?;
    store.set(
        "second".to_string(),
        Box::new(SimpleHandler("World".to_string())),
    )?;

    let mut results = Vec::new();
    store.apply(|key, handler| {
        let result = handler.handle()?;
        results.push((key.clone(), result));
        Ok(())
    })?;

    // Sort for stable testing
    results.sort();
    assert_eq!(
        results,
        vec![
            ("first".to_string(), "Hello".to_string()),
            ("second".to_string(), "World".to_string()),
        ]
    );

    Ok(())
}

#[test]
fn test_apply_empty() -> Result<(), MapError> {
    let store = TypeMapV::<String, i32>::new();

    // Applying to empty store should succeed
    store.apply(|_, _| -> Result<(), MapError> {
        panic!("This closure should never be called!");
    })?;

    Ok(())
}

#[test]
fn test_error_handling() {
    let store = TypeMapV::<String, i32>::new();

    // Test key not found
    match store.get(&"nope".to_string()) {
        Err(MapError::KeyNotFound(_)) => (),
        _ => panic!("Expected KeyNotFound error"),
    }

    // Add something so we have data to iterate over
    store.set("test".to_string(), 42).unwrap();

    // Now test error propagation from apply
    let err = store.apply(|_, _| Err(MapError::KeyNotFound("test".to_string())));
    assert!(matches!(err, Err(MapError::KeyNotFound(_))));
}

#[test]
fn test_thread_safety() -> Result<(), MapError> {
    use std::sync::Arc;
    use std::thread;

    let store = Arc::new(TypeMapV::<String, i32>::new());
    let store2 = Arc::clone(&store);

    // Initialize counter
    store.set("counter".to_string(), 0)?;

    // Spawn thread that increments by 1s
    let handle = thread::spawn(move || -> Result<(), MapError> {
        for _i in 0..100 {
            // Get current value and increment
            let current = store2.get(&"counter".to_string())?;
            store2.set("counter".to_string(), current + 1)?;
        }
        Ok(())
    });

    // Main thread increments by 1s too
    for _i in 0..100 {
        let current = store.get(&"counter".to_string())?;
        store.set("counter".to_string(), current + 1)?;
    }

    // Wait for other thread
    handle.join().unwrap()?;

    // We should have at least 100 total increments
    let final_value = store.get(&"counter".to_string())?;
    assert!(final_value > 0, "Counter should be greater than 0");
    assert!(final_value <= 200, "Counter should not exceed 200");

    Ok(())
}

#[test]
fn test_clone_shares_state() -> Result<(), MapError> {
    let store = TypeMapV::<String, i32>::new();
    let cloned = store.clone();

    store.set("test".to_string(), 42)?;
    assert_eq!(cloned.get(&"test".to_string())?, 42);

    Ok(())
}

#[test]
fn test_default_is_empty() -> Result<(), MapError> {
    let store = TypeMapV::<String, i32>::default();
    assert!(store.is_empty()?);
    Ok(())
}

#[test]
fn test_with() -> Result<(), MapError> {
    let store = TypeMapV::<String, Vec<i32>>::new();
    store.set("test".to_string(), vec![1, 2, 3])?;

    // Test reading without cloning
    let length = store.with(&"test".to_string(), |v| v.len())?;
    assert_eq!(length, 3);

    // Test computing derived value
    let sum = store.with(&"test".to_string(), |v| v.iter().sum::<i32>())?;
    assert_eq!(sum, 6);

    // Test key not found
    let err = store.with(&"nope".to_string(), |_: &Vec<i32>| ());
    assert!(matches!(err, Err(MapError::KeyNotFound(_))));

    Ok(())
}

#[test]
fn test_with_mut() -> Result<(), MapError> {
    let store = TypeMapV::<String, Vec<i32>>::new();
    store.set("test".to_string(), vec![1, 2, 3])?;

    // Test modifying in place
    let new_len = store.with_mut(&"test".to_string(), |v| {
        v.push(4);
        v.len()
    })?;
    assert_eq!(new_len, 4);
    assert_eq!(store.get(&"test".to_string())?, vec![1, 2, 3, 4]);

    // Test more complex modification
    store.with_mut(&"test".to_string(), |v| {
        v.sort_by(|a, b| b.cmp(a));  // reverse sort
    })?;
    assert_eq!(store.get(&"test".to_string())?, vec![4, 3, 2, 1]);

    // Test key not found
    let err = store.with_mut(&"nope".to_string(), |_: &mut Vec<i32>| ());
    assert!(matches!(err, Err(MapError::KeyNotFound(_))));

    Ok(())
}
