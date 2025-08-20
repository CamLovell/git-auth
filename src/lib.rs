use anyhow::Context;
use keyring::Entry;
use std::{
    collections::HashMap,
    io::{self, Read},
};

pub mod db;
pub mod github;

// TODO: Some of the contexts could be simplified to expects

#[allow(dead_code)] // Complains because it isn't used anywhere in lib.rs
#[derive(Debug)]
pub struct Request {
    host: String,
    protocol: String,
    path: Option<String>,
}

impl Request {
    pub fn from_stdin() -> anyhow::Result<Self> {
        let mut buf = String::new();
        // TODO: Proper error handling!
        let _ = io::stdin().read_to_string(&mut buf)?;
        let stdin_info: HashMap<String, String> = buf
            .lines()
            .filter_map(|l| {
                l.split_once("=")
                    .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
            })
            .collect();

        Ok(Request {
            protocol: stdin_info
                .get("protocol")
                .context("protocol not provided")?
                .to_string(),
            host: stdin_info
                .get("host")
                .context("host not provided")?
                .to_string(),
            path: stdin_info.get("path").cloned(),
        })
    }
}

pub struct Login {
    pub username: String,
    pub host: String,
    pub email: Option<String>,
    entry: Entry,
}

impl Login {
    pub fn new(username: String, host: String, email: Option<String>) -> anyhow::Result<Self> {
        let entry = Entry::new("git-auth", &format!("{}@{}", username, host))?;
        Ok(Self {
            username,
            host,
            email,
            entry,
        })
    }
    pub fn get_password(&self) -> anyhow::Result<String> {
        Ok(self.entry.get_password()?)
    }
    pub fn set_password(&self, password: &str) -> anyhow::Result<()> {
        self.entry.set_password(password)?;
        Ok(())
    }
}

pub fn send_creds(creds: &Login) -> anyhow::Result<()> {
    println!("username={}", creds.username);
    println!("password={}", creds.get_password()?);
    println!("quit=1");
    Ok(())
}
