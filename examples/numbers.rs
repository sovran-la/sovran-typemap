use sovran_typemap::{StoreError, TypeStore};

// Example trait: ThreadSafeNumber
trait ThreadSafeNumber {
    fn get_value(&self) -> i64;
    fn set_value(&mut self, value: i64);
}

struct IntegerValue {
    value: i64,
}

impl ThreadSafeNumber for IntegerValue {
    fn get_value(&self) -> i64 {
        self.value
    }
    fn set_value(&mut self, value: i64) {
        self.value = value;
    }
}

struct DoubledValue {
    value: i64,
}

impl ThreadSafeNumber for DoubledValue {
    fn get_value(&self) -> i64 {
        self.value * 2
    }
    fn set_value(&mut self, value: i64) {
        self.value = value / 2;
    }
}

fn main() -> Result<(), StoreError> {
    // Create a store
    let store: TypeStore<String> = TypeStore::new();

    // Store numbers with proper error handling
    store.set("num1".to_string(), IntegerValue { value: 42 })?;
    store.set("num2".to_string(), DoubledValue { value: 21 })?; // Will appear as 42 when read

    // Get numbers using with() and error handling
    match store.with(&"num1".to_string(), |num: &IntegerValue| {
        println!("Value 1: {}", num.get_value());
    }) {
        Ok(_) => println!("Successfully read IntegerValue"),
        Err(e) => println!("Error reading IntegerValue: {}", e),
    }

    // Alternative pattern using ? operator for early return
    store.with(&"num2".to_string(), |num: &DoubledValue| {
        println!("Value 2: {}", num.get_value());
    })?;

    // Update a number using with_mut()
    store.with_mut(&"num1".to_string(), |num: &mut IntegerValue| {
        num.set_value(100);
        println!("New value 1: {}", num.get_value());
    })?;

    // Get updated value using with()
    let value1 = store.with(&"num1".to_string(), |num: &IntegerValue| num.get_value())?;
    println!("Retrieved value 1: {}", value1);

    // Demonstrate safer remove with pattern matching
    match store.remove(&"num1".to_string()) {
        Ok(true) => println!("Successfully removed value 1"),
        Ok(false) => println!("Value 1 didn't exist"),
        Err(e) => println!("Error removing value 1: {}", e),
    }

    // Try accessing the removed value (should fail)
    match store.with(&"num1".to_string(), |_: &IntegerValue| {}) {
        Ok(_) => println!("This shouldn't happen - value 1 should be gone"),
        Err(StoreError::KeyNotFound(key)) => {
            println!("Correctly detected key ({}) was removed", key)
        }
        Err(e) => println!("Unexpected error: {}", e),
    }

    // Check store size
    match store.len() {
        Ok(count) => println!("Store has {} items", count),
        Err(e) => println!("Error getting store size: {}", e),
    }

    // Demonstrate is_empty
    let is_empty = store.is_empty()?;
    println!("Store is empty: {}", is_empty);

    Ok(())
}
