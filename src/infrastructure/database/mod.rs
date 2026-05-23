pub mod sqlite_repo;
pub mod auth_provider;

use rusqlite::Connection;
use anyhow::Result;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

pub fn init_db(path: &str) -> Result<Connection> {
    let conn = Connection::open(path)?;
    let schema = include_str!("schema.sql");
    conn.execute_batch(schema)?;
    Ok(conn)
}

pub fn init_db_pool(path: &str, max_size: u32) -> Result<Pool<SqliteConnectionManager>> {
    let manager = SqliteConnectionManager::file(path);
    let pool = Pool::builder()
        .max_size(max_size)
        .build(manager)?;
    
    // Initialize schema on first connection
    let conn = pool.get()?;
    let schema = include_str!("schema.sql");
    conn.execute_batch(schema)?;
    
    Ok(pool)
}

// Transaction trait for repositories
pub trait Transactional {
    fn execute_transaction<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&rusqlite::Connection) -> Result<R>;
}
