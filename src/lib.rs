use keyring::Entry;
use std::{
    collections::HashMap,
    fmt::Display,
    io::{self, Read},
};

pub mod db;
pub mod error;
pub mod github;

use error::GitError;

// TODO: Some of the contexts could be simplified to expects

#[allow(dead_code)] // Complains because it isn't used anywhere in lib.rs
#[derive(Debug)]
pub struct Request {
    pub host: String,
    protocol: String,
    pub owner: String,
}

impl Request {
    pub fn from_stdin() -> Result<Self, GitError> {
        let mut buf = String::new();
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
                .ok_or(GitError::MissingInfo(String::from("protocol")))?
                .to_string(),
            host: stdin_info
                .get("host")
                .ok_or(GitError::MissingInfo(String::from("host")))?
                .to_string(),
            owner: stdin_info
                .get("path")
                .ok_or(GitError::MissingInfo(String::from("path")))?
                .split_once("/")
                .map(|s| s.0)
                .ok_or(GitError::InvalidInfo(String::from(
                    "Provided path not complete",
                )))?
                .to_string(),
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

    pub fn get_password(&self) -> keyring::Result<String> {
        self.entry().get_password()
    }
    pub fn set_password(&self, password: &str) -> keyring::Result<()> {
        self.entry().set_password(password)
    }
    pub fn delete_password(&self) -> keyring::Result<()> {
        self.entry().delete_credential()
    }
}

impl Display for Login {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.email {
            Some(email) => write!(f, "{}, ({})", self.username, email),
            None => write!(f, "{}", self.username),
        }
    }
}

pub fn send_creds(creds: &Login) -> keyring::Result<()> {
    println!("username={}", creds.username);
    println!("password={}", creds.get_password()?);
    println!("quit=1");
    Ok(())
}
