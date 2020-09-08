use diesel::{ r2d2::ConnectionManager, SqliteConnection };

pub type DbManager = ConnectionManager<SqliteConnection>;

pub fn get_manager(db_path: &str) -> DbManager {
    let conn_manager: ConnectionManager<SqliteConnection> =
        ConnectionManager::new(db_path);
    
    conn_manager
}
