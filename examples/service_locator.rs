//! Demonstrates using TypeStore as a service locator / dependency injection container.
//!
//! TypeStore is perfect for DI because:
//! - Type IS the key - no string keys to manage
//! - Only one instance per type - natural singleton pattern
//! - Thread-safe - can be shared across the application
//!
//! Run with: cargo run --example service_locator

use sovran_typemap::{MapError, TypeStore};
use std::sync::Arc;

fn main() -> Result<(), MapError> {
    // Create our service container
    let services = Arc::new(TypeStore::new());

    // Register services - type is the key, no strings needed
    services.set(DatabaseConfig {
        host: "localhost".to_string(),
        port: 5432,
        database: "myapp".to_string(),
    })?;

    services.set(AppConfig {
        name: "MyApp".to_string(),
        debug: true,
        max_connections: 100,
    })?;

    services.set(Logger::new("app"))?;

    // Create components that depend on services
    let user_service = UserService::new(Arc::clone(&services));
    let order_service = OrderService::new(Arc::clone(&services));

    // Use the services
    user_service.create_user("alice")?;
    user_service.create_user("bob")?;
    order_service.create_order("alice", "Widget")?;

    // Services can be modified
    services.with_mut::<AppConfig, _, _>(|cfg| {
        cfg.debug = false;
        println!("Debug mode disabled");
    })?;

    // Check final state
    println!("\nFinal configuration:");
    services.with::<AppConfig, _, _>(|cfg| {
        println!("  App: {}", cfg.name);
        println!("  Debug: {}", cfg.debug);
        println!("  Max connections: {}", cfg.max_connections);
    })?;

    services.with::<DatabaseConfig, _, _>(|cfg| {
        println!("  Database: {}:{}/{}", cfg.host, cfg.port, cfg.database);
    })?;

    Ok(())
}

// ============================================================================
// Configuration types - stored in TypeStore
// ============================================================================

#[derive(Clone, Debug)]
struct DatabaseConfig {
    host: String,
    port: u16,
    database: String,
}

#[derive(Clone, Debug)]
struct AppConfig {
    name: String,
    debug: bool,
    max_connections: u32,
}

#[derive(Clone, Debug)]
struct Logger {
    prefix: String,
}

impl Logger {
    fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
        }
    }

    fn log(&self, message: &str) {
        println!("[{}] {}", self.prefix, message);
    }
}

// ============================================================================
// Services that consume dependencies from TypeStore
// ============================================================================

struct UserService {
    services: Arc<TypeStore>,
}

impl UserService {
    fn new(services: Arc<TypeStore>) -> Self {
        Self { services }
    }

    fn create_user(&self, username: &str) -> Result<(), MapError> {
        // Access logger from service container
        self.services.with::<Logger, _, _>(|logger| {
            logger.log(&format!("Creating user: {}", username));
        })?;

        // Access database config
        self.services.with::<DatabaseConfig, _, _>(|db| {
            println!(
                "  -> Would insert into {}.users on {}:{}",
                db.database, db.host, db.port
            );
        })?;

        // Check if debug mode
        self.services.with::<AppConfig, _, _>(|cfg| {
            if cfg.debug {
                println!("  -> [DEBUG] User {} created successfully", username);
            }
        })?;

        Ok(())
    }
}

struct OrderService {
    services: Arc<TypeStore>,
}

impl OrderService {
    fn new(services: Arc<TypeStore>) -> Self {
        Self { services }
    }

    fn create_order(&self, user: &str, item: &str) -> Result<(), MapError> {
        self.services.with::<Logger, _, _>(|logger| {
            logger.log(&format!("Creating order: {} for {}", item, user));
        })?;

        self.services.with::<DatabaseConfig, _, _>(|db| {
            println!(
                "  -> Would insert into {}.orders on {}:{}",
                db.database, db.host, db.port
            );
        })?;

        Ok(())
    }
}
