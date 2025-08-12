use std::env;
use std::fs;
use std::io::{self, Write};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 || args[1] != "get" {
        return;
    }

    let mut buffer = String::new();
    let mut file = fs::File::create(args[1].clone() + ".txt").unwrap();

    loop {
        let res = io::stdin().read_line(&mut buffer);
        match res {
            Ok(_) => {
                if buffer.is_empty() {
                    break;
                }
                let _ = file.write_all(buffer.as_bytes());
                buffer.clear();
                let pass_file = fs::read_to_string("pass.txt").unwrap();
                let pass = pass_file.lines().next().unwrap(); // Only want the first line

                let _ = io::stdout().write_all(b"username=CamLovell\n");
                let _ = io::stdout().write_all(format!("password={}\n", pass).as_bytes());
                let _ = io::stdout().write_all(b"quit=1\n");
            }
            Err(_) => println!("FAILED"),
        }
    }
}
