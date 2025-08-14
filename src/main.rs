use std::io::{self, Read, Write};
use std::{env, fs, path};

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
        git_auth::get_oauth();
    }

    let res = io::stdin().read_to_end(&mut buffer);
    match res {
        Ok(_) => {
            if !buffer.is_empty() {
                let _ = stdout_file.write_all(&buffer);
                buffer.clear();

                git_auth::send_creds();
            }
        }
        Err(_) => println!("FAILED"),
    }
}
