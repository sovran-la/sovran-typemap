use sovran_typemap::{MapError, TypeMap};

// Example trait: Animal
trait Animal {
    fn make_sound(&self) -> &str;
}

struct Dog {
    name: String,
}

impl Animal for Dog {
    fn make_sound(&self) -> &str {
        "Woof!"
    }
}

struct Cat {
    name: String,
}

impl Animal for Cat {
    fn make_sound(&self) -> &str {
        "Meow!"
    }
}

fn main() -> Result<(), MapError> {
    // Create a store
    let store: TypeMap<String> = TypeMap::new();

    // Store different animal types
    store.set(
        "dog1".to_string(),
        Dog {
            name: "Rover".to_string(),
        },
    )?;
    store.set(
        "cat1".to_string(),
        Cat {
            name: "Whiskers".to_string(),
        },
    )?;

    // Check if key exists
    if store.contains_key(&"dog1".to_string())? {
        println!("Dog exists in store");
    }

    // List all keys
    println!("Keys in store: {:?}", store.keys()?);

    // Get the dog and cat with proper error handling
    match store.with(&"dog1".to_string(), |dog: &Dog| {
        println!("Dog named {} says: {}", dog.name, dog.make_sound());
    }) {
        Ok(_) => println!("Successfully accessed dog"),
        Err(MapError::KeyNotFound(key)) => println!("{} not found in store", key),
        Err(MapError::TypeMismatch) => println!("Value is not a Dog"),
        Err(MapError::LockError) => println!("Failed to acquire lock"),
    }

    // Alternative pattern using if let for concise code
    if let Ok(_) = store.with(&"cat1".to_string(), |cat: &Cat| {
        println!("Cat named {} says: {}", cat.name, cat.make_sound());
    }) {
        println!("Successfully accessed cat");
    } else {
        println!("Failed to access cat");
    }

    // More explicit pattern matching for Cat
    match store.with(&"cat1".to_string(), |cat: &Cat| {
        format!("Cat named {} says: {}", cat.name, cat.make_sound())
    }) {
        Ok(message) => println!("{}", message),
        Err(e) => println!("Error accessing cat: {}", e),
    }

    // Attempting to access with incorrect type (intentional error)
    match store.with(&"dog1".to_string(), |_cat: &Cat| {
        // This should fail with TypeMismatch
    }) {
        Ok(_) => println!("This shouldn't happen"),
        Err(MapError::TypeMismatch) => println!("Correctly detected type mismatch"),
        Err(e) => println!("Unexpected error: {}", e),
    }

    Ok(())
}
