use serde_json::Value;
use std::{
    collections::HashMap,
    fs,
    io::{self, Read, Write},
    thread,
    time::Duration,
};

#[allow(dead_code)] // Complains because it isn't used anywhere in lib.rs
#[derive(Debug)]
pub struct GitRequest {
    host: String,
    protocol: String,
    path: Option<String>,
}

impl GitRequest {
    pub fn from_stdin() -> Self {
        let mut buf = String::new();
        // TODO: Proper error handling!
        let _ = io::stdin().read_to_string(&mut buf).unwrap();
        let stdin_info: HashMap<String, String> = buf
            .lines()
            .map(|l| {
                let (k, v) = l.split_once("=").unwrap();
                (k.trim().to_string(), v.trim().to_string())
            })
            .collect();

        GitRequest {
            protocol: stdin_info.get("protocol").unwrap().to_string(),
            host: stdin_info.get("host").unwrap().to_string(),
            path: stdin_info.get("path").cloned(),
        }
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

pub fn get_oauth() -> Creds {
    let client_id = fs::read_to_string("id.txt").unwrap();
    let client = reqwest::blocking::Client::new();
    let params = [("scope", "repo read:user"), ("client_id", client_id.trim())];
    let response = client
        .post("https://github.com/login/device/code")
        .header("Accept", "application/vnd.github+json ")
        .form(&params)
        .send()
        .unwrap();
    let txt = response.text().unwrap();
    eprintln!("CODE: {:?}", txt);
    let data: Value = serde_json::from_str(&txt).unwrap();

    let device_code = data["device_code"].as_str().unwrap();
    eprintln!("{}", data["verification_uri"].as_str().unwrap());
    eprintln!("{device_code}");
    let poll_params = [
        ("device_code", device_code),
        ("client_id", client_id.trim()),
        ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
    ];
    loop {
        let res = client
            .post("https://github.com/login/oauth/access_token")
            .header("Accept", "application/vnd.github+json")
            .form(&poll_params)
            .send()
            .unwrap();
        let txt = res.text().unwrap();
        let data: HashMap<String, Value> = serde_json::from_str(&txt).unwrap();
        eprintln!("{:?}", data);
        if let Some(token) = data.get("access_token") {
            let mut file = fs::File::create("pass.txt").unwrap();
            let token_str = token.as_str().unwrap();
            let _ = file.write_all(token_str.as_bytes());
            let auth = format!("token {token_str}");
            let res = client
                .get("https://api.github.com/user")
                .header("User-Agent", "git-auth")
                .header("Accept", "application/vnd.github+json")
                .header("Authorization", auth)
                .send()
                .unwrap();
            let txt = res.text().unwrap();
            let data: HashMap<String, Value> = serde_json::from_str(&txt).unwrap();
            eprintln!("{:?}", data["login"].as_str());
            return Creds {
                username: data["login"].as_str().unwrap().to_string(),
                password: token_str.to_string(),
            };
        }
        thread::sleep(Duration::from_secs(5));
    }
}
