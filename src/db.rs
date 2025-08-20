use crate::{Login, Request};
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
        "CREATE TABLE IF NOT EXISTS requests (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            protocol TEXT NOT NULL,
            host TEXT NOT NULL,
            path TEXT,
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

pub fn add_request(conn: &Connection, request: &Request, user_id: &i64) -> Result<i64> {
    conn.execute(
        "INSERT INTO requests (protocol, path, host, user_id) VALUES (?1, ?2, ?3, ?4)",
        params![request.protocol, request.path, request.host, user_id],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn fetch_login(conn: &Connection, request: &Request) -> anyhow::Result<Login> {
    let (username, email) = conn.query_row(
        "
        SELECT l.username, l.email
        FROM requests r
        JOIN logins l ON r.user_id = l.id
        WHERE r.host = ?1
          AND r.path = ?2
          AND r.protocol = ?3
        ",
        params![request.host, request.path, request.protocol],
        |row| Ok((row.get("username")?, row.get("email")?)),
    )?;
    Login::new(username, request.host.clone(), email)
}
