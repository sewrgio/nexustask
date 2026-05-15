pub mod sqlite_repo;
pub mod auth_provider;

use rusqlite::Connection;
use anyhow::Result;

pub fn init_db(path: &str) -> Result<Connection> {
    let conn = Connection::open(path)?;
    let schema = include_str!("schema.sql");
    conn.execute_batch(schema)?;
    Ok(conn)
}
