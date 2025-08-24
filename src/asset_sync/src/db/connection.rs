//! SQLite connection helpers.
//!
//! Provides [`connect_sqlite`] that opens a connection and applies recommended PRAGMAs
//! for local development: WAL journaling, foreign_keys=ON, and a 5000ms busy_timeout.
//!
//! Example:
//! ```no_run
//! use asset_sync::db::connection::connect_sqlite;
//!
//! let path = std::env::temp_dir().join("asset_sync_example.db");
//! let _conn = connect_sqlite(path.to_str().unwrap()).expect("open sqlite");
//! ```

use diesel::{Connection, RunQueryDsl, SqliteConnection, sql_query};

/// Open a SQLite connection and apply conneciton-wide PRAGMAs.
pub fn connect_sqlite(database_url: &str) -> anyhow::Result<SqliteConnection> {
    let mut conn = SqliteConnection::establish(database_url)?;

    // Better read concurrency + nicer dev ergomonics
    sql_query("PRAGMA journal_mode=WAL;").execute(&mut conn)?;
    sql_query("PRAGMA foreign_keys=ON;").execute(&mut conn)?;
    sql_query("PRAGMA busy_timeout=5000;").execute(&mut conn)?;
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::QueryableByName;
    use diesel::connection::SimpleConnection;
    use diesel::sql_types::{Integer, Text};
    use tempfile::NamedTempFile;

    #[derive(QueryableByName, Debug)]
    struct JournalMode {
        #[diesel(sql_type = Text)]
        journal_mode: String,
    }

    #[derive(QueryableByName, Debug)]
    struct ForeignKeys {
        #[diesel(sql_type = Integer)]
        foreign_keys: i32,
    }

    #[derive(QueryableByName, Debug)]
    struct BusyTimeout {
        #[diesel(sql_type = Integer)]
        busy_timeout: i32,
    }

    #[test]
    fn pragmas_applied() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_string_lossy().to_string();

        let mut conn = connect_sqlite(&path).expect("connect_sqlite");

        // Verify WAL journaling (value is lowercased by SQLite)
        let jm: JournalMode = sql_query("PRAGMA journal_mode;")
            .get_result(&mut conn)
            .expect("journal_mode");
        assert_eq!(jm.journal_mode.to_lowercase(), "wal");

        // Verify foreign_keys=ON
        let fk: ForeignKeys = sql_query("PRAGMA foreign_keys;")
            .get_result(&mut conn)
            .expect("foreign_keys");
        assert_eq!(fk.foreign_keys, 1);

        // Verify busy_timeout=5000ms
        let bt: BusyTimeout = sql_query("PRAGMA busy_timeout;")
            .get_result(&mut conn)
            .expect("busy_timeout");
        assert_eq!(bt.busy_timeout, 5000);
    }

    #[test]
    fn basic_crud_works() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_string_lossy().to_string();

        let mut conn = connect_sqlite(&path).expect("connect_sqlite");

        conn.batch_execute("CREATE TABLE t(x INTEGER);").unwrap();
        sql_query("INSERT INTO t (x) VALUES (1), (2), (3);")
            .execute(&mut conn)
            .unwrap();

        #[derive(QueryableByName, Debug)]
        struct Cnt {
            #[diesel(sql_type = Integer)]
            c: i32,
        }

        let cnt: Cnt = sql_query("SELECT COUNT(*) AS c FROM t;")
            .get_result(&mut conn)
            .unwrap();
        assert_eq!(cnt.c, 3);
    }
}
