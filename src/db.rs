use diesel::{ r2d2::ConnectionManager, SqliteConnection };
use diesel_migrations::embed_migrations;


embed_migrations!("migrations");

pub type DbManager = ConnectionManager<SqliteConnection>;

pub fn get_manager(db_path: &str) -> DbManager {
    let conn_manager: ConnectionManager<SqliteConnection> =
        ConnectionManager::new(db_path);
    
    conn_manager
}
