// src/trait_tests.rs
#[cfg(test)]
mod tests {
    use crate::traits::TypeMapValue;
    use crate::{MapError, TraitTypeMap};
    use std::any::{Any, TypeId};

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

    // Add conversion implementation for Dog
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

    // Add conversion implementation for Cat
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
        store.set_trait::<dyn Animal, _>("dog".to_string(), Dog {
            name: "Rover".to_string(),
            breed: "Golden Retriever".to_string(),
        })?;


        store.set_trait::<dyn Animal, _>("cat".to_string(), Cat {
            name: "Whiskers".to_string(),
            lives: 9,
        })?;

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
        store.set_trait::<dyn Animal, _>("cat".to_string(), Cat {
            name: "Whiskers".to_string(),
            lives: 9,
        })?;

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
        store.set_trait::<dyn Animal, _>("pet".to_string(), Dog {
            name: "Rover".to_string(),
            breed: "Golden Retriever".to_string(),
        }).unwrap();

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
}