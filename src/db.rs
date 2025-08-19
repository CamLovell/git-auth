use crate::Login;
use rusqlite::{Connection, Result, params};

pub fn open() -> Result<Connection> {
    let conn = Connection::open("test.db")?;

    conn.execute("PRAGMA foreign_keys = ON", ())?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS logins (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL,
            email TEXT,
            host TEXT NOT NULL
        )
        ",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS loactions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            protocol TEXT NOT NULL,
            host TEXT NOT NULL,
            owner TEXT,
            user_id INTEGER,
            FOREIGN KEY (user_id) REFERENCES logins (id)
        )
        ",
        (),
    )?;

    Ok(conn)
}

pub fn add_login(conn: &Connection, login: &Login) -> Result<i64> {
    conn.execute(
        "INSERT INTO logins (username, email, host) VALUES (?1, ?2, ?3)",
        params![login.username, login.email, login.host],
    )?;
    Ok(conn.last_insert_rowid())
}
