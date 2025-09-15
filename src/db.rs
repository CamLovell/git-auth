use crate::{Login, Request, error::DatabaseError};
use rusqlite::{Connection, params};
use std::{env, fs};

pub fn open() -> Result<Connection, DatabaseError> {
    let path = env::home_dir()
        .ok_or(DatabaseError::Path)?
        .join(".local/share/git-auth");

    if !path.exists() {
        fs::create_dir_all(&path)?;
    }

    let conn = Connection::open(path.join("creds.db"))?;

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
            valid BOOLEAN NOT NULL DEFAULT 0,
            user_id INTEGER,
            FOREIGN KEY (user_id) REFERENCES logins (id)
        )
        ",
        (),
    )?;

    Ok(conn)
}

pub fn add_login(conn: &Connection, login: &Login) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO logins (username, email, host) VALUES (?1, ?2, ?3)",
        params![login.username, login.email, login.host],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn validate_request(conn: &Connection, request: &Request, valid: bool) -> rusqlite::Result<()> {
    conn.execute(
        "
        UPDATE requests
        SET valid = ?1
        WHERE host = ?2
            AND path like ?3
            AND protocol = ?4
        ",
        params![
            valid,
            request.host,
            format!("{}%", request.path_parent()),
            request.protocol
        ],
    )?;
    Ok(())
}

pub fn add_request(conn: &Connection, request: &Request, user_id: &i64) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO requests (protocol, path, host, user_id) VALUES (?1, ?2, ?3, ?4)",
        params![
            request.protocol,
            request.path_parent(),
            request.host,
            user_id
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn fetch_login(conn: &Connection, request: &Request) -> rusqlite::Result<(Login, bool)> {
    conn.query_row(
        "
        SELECT l.username, l.email, r.valid
        FROM requests r
        JOIN logins l ON r.user_id = l.id
        WHERE r.host = ?1
          AND r.path LIKE ?2
          AND r.protocol = ?3
        ",
        params![
            request.host,
            format!("{}%", request.path_parent()),
            request.protocol
        ],
        |row| {
            Ok((
                Login::new(
                    row.get("username")?,
                    request.host.clone(),
                    row.get("email")?,
                ),
                row.get("valid")?,
            ))
        },
    )
}

pub fn fetch_available_logins(
    conn: &Connection,
    request: &Request,
) -> rusqlite::Result<Vec<Login>> {
    let mut stmt = conn.prepare("SELECT username, email FROM logins WHERE host = ?1")?;
    stmt.query_map(params![request.host], |row| {
        Ok(Login::new(
            row.get("username")?,
            request.host.clone(),
            row.get("email")?,
        ))
    })?
    .collect()
}

pub fn fetch_all_logins(conn: &Connection) -> rusqlite::Result<Vec<Login>> {
    let mut stmt = conn.prepare("SELECT username, email, host FROM logins")?;
    stmt.query_map((), |row| {
        Ok(Login::new(
            row.get("username")?,
            row.get("host")?,
            row.get("email")?,
        ))
    })?
    .collect()
}
