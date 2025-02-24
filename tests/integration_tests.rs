use sovran_typemap::TypeStore;

#[test]
fn test_basic_operations() {
    let store: TypeStore<String> = TypeStore::new();

    // Store a value
    store.set("key".to_string(), 42i32).unwrap();

    // Check if key exists
    assert!(store.contains_key(&"key".to_string()).unwrap());

    // Get the value
    let value = store.get::<i32>(&"key".to_string()).unwrap();
    assert_eq!(*value, 42);

    // Update the value
    let mut value = store.get_mut::<i32>(&"key".to_string()).unwrap();
    *value = 100;

    // Check the updated value
    let value = store.get::<i32>(&"key".to_string()).unwrap();
    assert_eq!(*value, 100);

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
    let result = store.get::<i32>(&"key".to_string());
    assert!(result.is_err());

    // Get it as the correct type
    let value = store.get::<String>(&"key".to_string()).unwrap();
    assert_eq!(*value, "hello");
}

#[test]
fn test_multiple_types() {
    let store: TypeStore<String> = TypeStore::new();

    // Store different types
    store.set("int".to_string(), 42i32).unwrap();
    store.set("string".to_string(), "hello".to_string()).unwrap();
    store.set("float".to_string(), 3.14f64).unwrap();

    // Get them back
    assert_eq!(*store.get::<i32>(&"int".to_string()).unwrap(), 42);
    assert_eq!(*store.get::<String>(&"string".to_string()).unwrap(), "hello");
    assert_eq!(*store.get::<f64>(&"float".to_string()).unwrap(), 3.14);

    // Get the keys
    let keys = store.keys().unwrap();
    assert_eq!(keys.len(), 3);
    assert!(keys.contains(&"int".to_string()));
    assert!(keys.contains(&"string".to_string()));
    assert!(keys.contains(&"float".to_string()));
}