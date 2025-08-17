use anyhow::{Context, anyhow};
use serde_json::Value;
use std::{
    collections::HashMap,
    fs,
    io::{self, Read},
    sync::mpsc::RecvTimeoutError,
    thread,
    time::{self, Duration},
};

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

pub struct Creds {
    username: String,
    password: String,
}

pub fn send_creds(creds: &Creds) {
    println!("username={}", creds.username);
    println!("password={}", creds.password);
    println!("quit=1");
}

pub fn get_oauth() -> anyhow::Result<Creds> {
    let client_id = fs::read_to_string("id.txt")?;
    let client_id = client_id.trim();
    let client = reqwest::blocking::Client::new();
    let params = [("scope", "repo read:user"), ("client_id", client_id)];
    let response: HashMap<String, Value> = client
        .post("https://github.com/login/device/code")
        .header("Accept", "application/vnd.github+json ")
        .form(&params)
        .send()?
        .json()?;

    let device_code = response["device_code"]
        .as_str()
        .context("device_code not found in server response")?;
    eprintln!(
        "Copy this code <{}> and follow the instructions here\n{}",
        response["user_code"]
            .as_str()
            .context("user_code not found in server resonse")?,
        response["verification_uri"]
            .as_str()
            .context("verification_uri not found in server resonse")?
    );
    let poll_params = [
        ("device_code", device_code),
        ("client_id", client_id),
        ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
    ];
    let loop_start = time::SystemTime::now();
    loop {
        let res: HashMap<String, Value> = client
            .post("https://github.com/login/oauth/access_token")
            .header("Accept", "application/vnd.github+json")
            .form(&poll_params)
            .send()?
            .json()?;
        match res.get("access_token") {
            Some(token) => {
                let token_str = token
                    .as_str()
                    .context("Failed to convert token to string")?;
                let res: HashMap<String, Value> = client
                    .get("https://api.github.com/user")
                    .header("User-Agent", "git-auth")
                    .header("Accept", "application/vnd.github+json")
                    .header("Authorization", format!("token {token_str}"))
                    .send()?
                    .json()?;
                return Ok(Creds {
                    username: res["login"]
                        .as_str()
                        .context("login not present in user request")?
                        .to_string(),
                    password: token_str.to_string(),
                });
            }
            None => {
                // TODO: Read expiry from initial api respinse
                if time::SystemTime::now()
                    .duration_since(loop_start)?
                    .as_secs()
                    > 900
                {
                    // TODO: This feels bad, fix it
                    eprintln!("Code has expired, exiting");
                    return Err(anyhow!(RecvTimeoutError::Timeout));
                }

                thread::sleep(Duration::from_secs(5));
            }
        }
    }
}
