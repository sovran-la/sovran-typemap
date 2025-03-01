#![allow(dead_code)]

use sovran_typemap::{MapError, TypeMap};
use std::collections::HashMap;
use std::sync::Arc;

/// Demonstrates using TypeMap for managing application state
fn main() -> Result<(), MapError> {
    // Create our application state store
    let app_state = Arc::new(TypeMap::<String>::new());

    // Initialize different parts of the application state
    initialize_app_state(&app_state)?;

    // Create modules that will use the shared state
    let user_module = UserModule::new(Arc::clone(&app_state));
    let config_module = ConfigModule::new(Arc::clone(&app_state));
    let stats_module = StatsModule::new(Arc::clone(&app_state));

    // Simulate some application activity

    // Add some users
    user_module.add_user(User {
        id: 1,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
        active: true,
    })?;

    user_module.add_user(User {
        id: 2,
        username: "bob".to_string(),
        email: "bob@example.com".to_string(),
        active: true,
    })?;

    // Update configuration
    config_module.set_theme("dark".to_string())?;
    config_module.set_language("en-US".to_string())?;

    // Record some statistics
    stats_module.record_page_view("home".to_string())?;
    stats_module.record_page_view("profile".to_string())?;
    stats_module.record_page_view("home".to_string())?;

    // Display application state
    print_app_state(&app_state)?;

    // Update a user
    user_module.deactivate_user(1)?;
    println!("\nAfter deactivating user 1:");

    // Display users after update
    app_state.with(&"users".to_string(), |users: &Vec<User>| {
        for user in users {
            println!(
                "User {}: {} <{}> - active: {}",
                user.id, user.username, user.email, user.active
            );
        }
    })?;

    Ok(())
}

/// Initialize all required application state
fn initialize_app_state(store: &TypeMap<String>) -> Result<(), MapError> {
    // Initialize users collection
    store.set("users".to_string(), Vec::<User>::new())?;

    // Initialize configuration with defaults
    let mut config = HashMap::new();
    config.insert("theme".to_string(), "light".to_string());
    config.insert("language".to_string(), "en-US".to_string());
    config.insert("notifications".to_string(), "enabled".to_string());
    store.set("config".to_string(), config)?;

    // Initialize statistics counters
    let page_views: HashMap<String, u32> = HashMap::new();
    store.set("page_views".to_string(), page_views)?;

    // Initialize application startup time
    store.set("startup_time".to_string(), chrono::Local::now())?;

    Ok(())
}

/// Print the current application state
fn print_app_state(store: &TypeMap<String>) -> Result<(), MapError> {
    println!("APPLICATION STATE:");
    println!("=================");

    // Print users
    println!("\nUSERS:");
    store.with(&"users".to_string(), |users: &Vec<User>| {
        for user in users {
            println!(
                "User {}: {} <{}> - active: {}",
                user.id, user.username, user.email, user.active
            );
        }
    })?;

    // Print configuration
    println!("\nCONFIGURATION:");
    store.with(&"config".to_string(), |config: &HashMap<String, String>| {
        for (key, value) in config {
            println!("{}: {}", key, value);
        }
    })?;

    // Print page view statistics
    println!("\nPAGE VIEWS:");
    store.with(&"page_views".to_string(), |views: &HashMap<String, u32>| {
        let mut total = 0;
        for (page, count) in views {
            println!("{}: {} views", page, count);
            total += count;
        }
        println!("Total page views: {}", total);
    })?;

    // Print application uptime
    store.with(
        &"startup_time".to_string(),
        |time: &chrono::DateTime<chrono::Local>| {
            let now = chrono::Local::now();
            let duration = now.signed_duration_since(*time);
            println!("\nApplication uptime: {} seconds", duration.num_seconds());
        },
    )?;

    Ok(())
}

// Data structures

#[derive(Clone, Debug)]
struct User {
    id: u64,
    username: String,
    email: String,
    active: bool,
}

// Application modules

struct UserModule {
    store: Arc<TypeMap<String>>,
}

impl UserModule {
    fn new(store: Arc<TypeMap<String>>) -> Self {
        Self { store }
    }

    fn add_user(&self, user: User) -> Result<(), MapError> {
        self.store
            .with_mut(&"users".to_string(), |users: &mut Vec<User>| {
                // Check if user already exists
                if users
                    .iter()
                    .any(|u| u.id == user.id || u.username == user.username)
                {
                    return;
                }
                users.push(user);
            })
    }

    fn get_user_by_id(&self, id: u64) -> Result<Option<User>, MapError> {
        self.store.with(&"users".to_string(), |users: &Vec<User>| {
            users.iter().find(|u| u.id == id).cloned()
        })
    }

    fn deactivate_user(&self, id: u64) -> Result<bool, MapError> {
        self.store
            .with_mut(&"users".to_string(), |users: &mut Vec<User>| {
                if let Some(user) = users.iter_mut().find(|u| u.id == id) {
                    user.active = false;
                    true
                } else {
                    false
                }
            })
    }
}

struct ConfigModule {
    store: Arc<TypeMap<String>>,
}

impl ConfigModule {
    fn new(store: Arc<TypeMap<String>>) -> Self {
        Self { store }
    }

    fn get_config(&self, key: &str) -> Result<Option<String>, MapError> {
        self.store
            .with(&"config".to_string(), |config: &HashMap<String, String>| {
                config.get(key).cloned()
            })
    }

    fn set_theme(&self, theme: String) -> Result<(), MapError> {
        self.store.with_mut(
            &"config".to_string(),
            |config: &mut HashMap<String, String>| {
                config.insert("theme".to_string(), theme);
            },
        )
    }

    fn set_language(&self, language: String) -> Result<(), MapError> {
        self.store.with_mut(
            &"config".to_string(),
            |config: &mut HashMap<String, String>| {
                config.insert("language".to_string(), language);
            },
        )
    }
}

struct StatsModule {
    store: Arc<TypeMap<String>>,
}

impl StatsModule {
    fn new(store: Arc<TypeMap<String>>) -> Self {
        Self { store }
    }

    fn record_page_view(&self, page: String) -> Result<(), MapError> {
        self.store.with_mut(
            &"page_views".to_string(),
            |views: &mut HashMap<String, u32>| {
                *views.entry(page).or_insert(0) += 1;
            },
        )
    }

    fn get_total_views(&self) -> Result<u32, MapError> {
        self.store
            .with(&"page_views".to_string(), |views: &HashMap<String, u32>| {
                views.values().sum()
            })
    }

    fn get_page_views(&self, page: &str) -> Result<u32, MapError> {
        self.store
            .with(&"page_views".to_string(), |views: &HashMap<String, u32>| {
                views.get(page).copied().unwrap_or(0)
            })
    }
}
