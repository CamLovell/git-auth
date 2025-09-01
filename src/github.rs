use crate::{Login, error::GithubError};
use colored::Colorize;
use serde_json::Value;
use std::{
    collections::HashMap,
    thread,
    time::{self, Duration},
};

pub fn get_login() -> Result<Login, GithubError> {
    let client_id = String::from("Ov23liAXHnUzobAF9AuF");
    let client = reqwest::blocking::Client::new();

    let device_code = get_device_code(&client, &client_id)?;
    let token = poll_for_auth(&client, &device_code, &client_id)?;
    let username = query_username(&client, &token)?;
    let email = query_email(&client, &token)?;

    let login = Login::new(username, String::from("github.com"), email);
    login.set_password(&token)?;
    Ok(login)
}

fn get_device_code(
    reqwest_client: &reqwest::blocking::Client,
    client_id: &str,
) -> Result<String, GithubError> {
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
        "Copy this code <{}> and follow the instructions at the link\n\t{}",
        response
            .get("user_code")
            .ok_or(GithubError::MissingField(String::from("user_code")))?
            .as_str()
            .ok_or(GithubError::InvalidField(String::from("user_code")))?
            .green()
            .bold(),
        response
            .get("verification_uri")
            .ok_or(GithubError::MissingField(String::from("verification_uri")))?
            .as_str()
            .ok_or(GithubError::InvalidField(String::from("verification_uri")))?
            .blue()
            .underline()
    );

    // TODO: Add expiry reading here so timeout is based on expiry
    Ok(response
        .get("device_code")
        .ok_or(GithubError::MissingField(String::from("device_code")))?
        .as_str()
        .ok_or(GithubError::InvalidField(String::from("device_code")))?
        .to_string())
}

fn poll_for_auth(
    reqwest_client: &reqwest::blocking::Client,
    device_code: &str,
    client_id: &str,
) -> Result<String, GithubError> {
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
                    .ok_or(GithubError::InvalidField(String::from("access_token")))?
                    .to_string());
            }
            None => {
                // TODO: Read expiry from initial api response
                if time::SystemTime::now()
                    .duration_since(loop_start)
                    .expect("now cannot be before loop_start")
                    .as_secs()
                    > 900
                {
                    eprintln!("Code has expired, exiting");
                    break Err(GithubError::Timeout(900));
                }
                thread::sleep(Duration::from_secs(5));
            }
        }
    }
}

fn query_username(
    reqwest_client: &reqwest::blocking::Client,
    token: &str,
) -> Result<String, GithubError> {
    let res: HashMap<String, Value> = reqwest_client
        .get("https://api.github.com/user")
        .header("User-Agent", "git-auth")
        .header("Accept", "application/vnd.github+json")
        .header("Authorization", format!("token {token}"))
        .send()?
        .json()?;
    Ok(res
        .get("login")
        .ok_or(GithubError::MissingField(String::from("login")))?
        .as_str()
        .ok_or(GithubError::InvalidField(String::from("login")))?
        .to_string())
}

fn query_email(
    reqwest_client: &reqwest::blocking::Client,
    token: &str,
) -> Result<Option<String>, GithubError> {
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
