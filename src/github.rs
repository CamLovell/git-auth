use crate::Login;
use anyhow::{Context, anyhow};
use serde_json::Value;
use std::{
    collections::HashMap,
    fs,
    sync::mpsc::RecvTimeoutError,
    thread,
    time::{self, Duration},
};

pub fn get_login() -> anyhow::Result<Login> {
    let client_id = fs::read_to_string("id.txt")?;
    let client_id = client_id.trim().to_string();
    let client = reqwest::blocking::Client::new();

    let device_code = get_device_code(&client, &client_id)?;
    let token = poll_for_auth(&client, &device_code, &client_id)?;
    let username = query_username(&client, &token)?;
    let email = query_email(&client, &token)?;

    Ok(Login {
        username,
        email,
        host: String::from("github.com"),
        password: token,
    })
}

fn get_device_code(
    reqwest_client: &reqwest::blocking::Client,
    client_id: &str,
) -> anyhow::Result<String> {
    let params = [
        ("scope", "repo read:user user:email"),
        ("client_id", client_id),
    ];
    let response: HashMap<String, Value> = reqwest_client
        .post("https://github.com/login/device/code")
        .header("Accept", "application/vnd.github+json ")
        .form(&params)
        .send()?
        .json()?;

    eprintln!(
        "Copy this code <{}> and follow the instructions here\n{}",
        response["user_code"]
            .as_str()
            .context("user_code not found in server resonse")?,
        response["verification_uri"]
            .as_str()
            .context("verification_uri not found in server resonse")?
    );

    Ok(response["device_code"]
        .as_str()
        .context("device_code not found in server response")?
        .to_string())
}

fn poll_for_auth(
    reqwest_client: &reqwest::blocking::Client,
    device_code: &str,
    client_id: &str,
) -> anyhow::Result<String> {
    let poll_params = [
        ("device_code", device_code),
        ("client_id", client_id),
        ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
    ];
    let loop_start = time::SystemTime::now();
    loop {
        let res: HashMap<String, Value> = reqwest_client
            .post("https://github.com/login/oauth/access_token")
            .header("Accept", "application/vnd.github+json")
            .form(&poll_params)
            .send()?
            .json()?;
        match res.get("access_token") {
            Some(token) => {
                break Ok(token
                    .as_str()
                    .context("Failed to convert token to string")?
                    .to_string());
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
                    break Err(anyhow!(RecvTimeoutError::Timeout));
                }
                thread::sleep(Duration::from_secs(5));
            }
        }
    }
}

fn query_username(
    reqwest_client: &reqwest::blocking::Client,
    token: &str,
) -> anyhow::Result<String> {
    let res: HashMap<String, Value> = reqwest_client
        .get("https://api.github.com/user")
        .header("User-Agent", "git-auth")
        .header("Accept", "application/vnd.github+json")
        .header("Authorization", format!("token {token}"))
        .send()?
        .json()?;
    Ok(res["login"]
        .as_str()
        .context("login not present in user request")?
        .to_string())
}

fn query_email(
    reqwest_client: &reqwest::blocking::Client,
    token: &str,
) -> anyhow::Result<Option<String>> {
    let res: Vec<Value> = reqwest_client
        .get("https://api.github.com/user/emails")
        .header("User-Agent", "git-auth")
        .header("Accept", "application/vnd.github+json")
        .header("Authorization", format!("token {token}"))
        .send()?
        .json()?;
    for email in res {
        if let Some(Value::Bool(true)) = email.get("primary") {
            return Ok(email
                .get("email")
                .and_then(|e| e.as_str().map(|e_str| e_str.to_string())));
        }
    }

    Ok(None)
}
