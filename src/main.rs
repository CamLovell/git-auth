use serde_json::Value;
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::{env, fs, path};
use std::{thread, time::Duration};

fn main() {
    eprintln!("Calling  git-auth");
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 || args[1] != "get" {
        return;
    }

    let mut buffer = Vec::new();
    let path_str = args[1].clone() + ".txt";
    let path = path::Path::new(&path_str);
    let mut stdout_file;
    if path.exists() {
        stdout_file = fs::File::options().append(true).open(path).unwrap();
        let _ = stdout_file.write_all(b"\n\n\n");
    } else {
        stdout_file = fs::File::create(path).unwrap();
    }
    if !path::Path::new("pass.txt").exists() {
        get_oauth();
    }

    let res = io::stdin().read_to_end(&mut buffer);
    match res {
        Ok(_) => {
            if !buffer.is_empty() {
                let _ = stdout_file.write_all(&buffer);
                buffer.clear();

                let pass_file = fs::read_to_string("pass.txt").unwrap();
                let pass = pass_file.lines().next().unwrap(); // Only want the first line

                let _ = io::stdout().write_all(b"username=CamLovell\n");
                let _ = io::stdout().write_all(format!("password={}\n", pass).as_bytes());
                let _ = io::stdout().write_all(b"quit=1\n");
            }
        }
        Err(_) => println!("FAILED"),
    }
}

fn get_oauth() {
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
