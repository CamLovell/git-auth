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
    pub host: String,
    protocol: String,
    pub path: Option<String>,
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

#[derive(Clone)]
pub struct Login {
    pub username: String,
    pub host: String,
    pub email: Option<String>,
}

impl Login {
    pub fn new(username: String, host: String, email: Option<String>) -> Self {
        Self {
            username,
            host,
            email,
        }
    }

    fn entry(&self) -> Entry {
        let identifier = format!("{}@{}", self.username, self.host);
        match Entry::new("git-auth", &identifier) {
            Ok(entry) => entry,
            Err(keyring::error::Error::TooLong(_, max)) => {
                Entry::new("git-auth", identifier.split_at(max as usize).0)
                    .expect("Expected entry creation success after handling length")
            }
            Err(err) => panic!("Unrecoverable error in login creation:\n{err}"),
        }
    }

    pub fn get_password(&self) -> anyhow::Result<String> {
        Ok(self.entry().get_password()?)
    }
    pub fn set_password(&self, password: &str) -> anyhow::Result<()> {
        self.entry().set_password(password)?;
        Ok(())
    }
}

pub fn send_creds(creds: &Login) -> anyhow::Result<()> {
    println!("username={}", creds.username);
    println!("password={}", creds.get_password()?);
    println!("quit=1");
    Ok(())
}
