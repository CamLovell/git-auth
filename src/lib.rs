use serde_json::Value;
use std::{
    collections::HashMap,
    fs,
    io::{self, Write},
    thread,
    time::Duration,
};

pub fn send_creds() {
    let pass_file = fs::read_to_string("pass.txt").unwrap();
    let pass = pass_file.lines().next().unwrap(); // Only want the first line

    let _ = io::stdout().write_all(b"username=CamLovell\n");
    let _ = io::stdout().write_all(format!("password={}\n", pass).as_bytes());
    let _ = io::stdout().write_all(b"quit=1\n");
}

pub fn get_oauth() {
    let client_id = fs::read_to_string("id.txt").unwrap();
    let client = reqwest::blocking::Client::new();
    let params = [("scope", "repo"), ("client_id", client_id.as_str())];
    let response = client
        .post("https://github.com/login/device/code")
        .header("Accept", "application/json")
        .form(&params)
        .send()
        .unwrap();
    let txt = response.text().unwrap();
    eprintln!("{:?}", txt);
    let data: Value = serde_json::from_str(&txt).unwrap();

    let device_code = data["device_code"].as_str().unwrap();
    eprintln!("{}", data["verification_uri"].as_str().unwrap());
    eprintln!("{device_code}");
    let poll_params = [
        ("device_code", device_code),
        ("client_id", client_id.as_str()),
        ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
    ];
    loop {
        let res = client
            .post("https://github.com/login/oauth/access_token")
            .header("Accept", "application/json")
            .form(&poll_params)
            .send()
            .unwrap();
        let txt = res.text().unwrap();
        let data: HashMap<String, Value> = serde_json::from_str(&txt).unwrap();
        if let Some(token) = data.get("access_token") {
            let mut file = fs::File::create("pass.txt").unwrap();
            let _ = file.write_all(
                token
                    .to_string()
                    .strip_prefix("\"")
                    .unwrap()
                    .strip_suffix("\"")
                    .unwrap()
                    .as_bytes(),
            );
            eprintln!("{}", token);
            break;
        }
        thread::sleep(Duration::from_secs(5));
    }
}
