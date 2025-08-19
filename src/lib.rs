use anyhow::Context;
use std::{
    collections::HashMap,
    io::{self, Read},
};

pub mod db;
pub mod github;

// TODO: Some of the contexts could be simplified to expects

#[allow(dead_code)] // Complains because it isn't used anywhere in lib.rs
#[derive(Debug)]
pub struct GitRequest {
    host: String,
    protocol: String,
    path: Option<String>,
}

impl GitRequest {
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

        Ok(GitRequest {
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
    pub password: String,
}

pub fn send_creds(creds: &Login) {
    println!("username={}", creds.username);
    println!("password={}", creds.password);
    println!("quit=1");
}
