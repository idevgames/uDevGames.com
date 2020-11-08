use diesel::{
    r2d2::{ConnectionManager, Pool, PooledConnection},
    SqliteConnection,
};
use diesel_migrations::embed_migrations;

embed_migrations!("migrations");

pub type DbManager = ConnectionManager<SqliteConnection>;
pub type DbPool = Pool<DbManager>;
pub type DbConn = PooledConnection<ConnectionManager<SqliteConnection>>;

// gets the pool.
pub fn get_pool(db_path: &str) -> DbPool {
    let conn_manager: ConnectionManager<SqliteConnection> =
        ConnectionManager::new(db_path);
    let pool = diesel::r2d2::Pool::builder()
        .max_size(2)
        .build(conn_manager)
        .unwrap();

    pool
}

/// migrates the db. panics if there is a migration failure.
pub fn migrate_db(manager: &DbPool) {
    let conn = manager.get().unwrap();
    embedded_migrations::run_with_output(&conn, &mut std::io::stdout())
        .unwrap();
}
