use crate::{Login, Request};
use anyhow::Context;
use rusqlite::{Connection, Result, params};
use std::{env, fs};

pub fn open() -> anyhow::Result<Connection> {
    let path = env::home_dir()
        .context("home is unknown")?
        .join(".local/share/git-auth/creds.db");

    if !path.parent().expect("parents must exits").exists() {
        fs::create_dir_all(path.parent().expect("parents must exits"))?;
    }

    let conn = Connection::open(path)?;

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

pub fn add_login(conn: &Connection, login: &Login) -> Result<i64> {
    conn.execute(
        "INSERT INTO logins (username, email, host) VALUES (?1, ?2, ?3)",
        params![login.username, login.email, login.host],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn validate_request(conn: &Connection, request: &Request, valid: bool) -> Result<usize> {
    conn.execute(
        "
        UPDATE requests
        SET valid = ?1
        WHERE host = ?2
            AND path = ?3
            AND protocol = ?4
        ",
        params![valid, request.host, request.path, request.protocol],
    )
}
pub fn add_request(conn: &Connection, request: &Request, user_id: &i64) -> Result<i64> {
    conn.execute(
        "INSERT INTO requests (protocol, path, host, user_id) VALUES (?1, ?2, ?3, ?4)",
        params![request.protocol, request.path, request.host, user_id],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn fetch_login(conn: &Connection, request: &Request) -> Result<Login> {
    conn.query_row(
        "
        SELECT l.username, l.email
        FROM requests r
        JOIN logins l ON r.user_id = l.id
        WHERE r.host = ?1
          AND r.path = ?2
          AND r.protocol = ?3
        ",
        params![request.host, request.path, request.protocol],
        |row| {
            Ok(Login::new(
                row.get("username")?,
                request.host.clone(),
                row.get("email")?,
            ))
        },
    )
}

pub fn fetch_available_logins(conn: &Connection, request: &Request) -> Result<Vec<Login>> {
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
