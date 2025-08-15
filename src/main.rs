use clap::{Parser, Subcommand};
use git_auth::{GitRequest, get_oauth, send_creds};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    // Get a credential
    Get,
    Store,
    Erase,
    Init,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Get => {
            // let git_request = GitRequest::from_stdin();
            // eprintln!("{:?}", git_request);
            let creds = get_oauth();
            send_creds(&creds);
        }
        Commands::Init => eprintln!("Initialising"),
        Commands::Store => eprintln!("Storeing"),
        Commands::Erase => eprintln!("Eraseing"),
    }
}

// Old Code (Keep bits to use later)

// use std::io::{self, Read, Write};
// use std::{env, fs, path};
// eprintln!("Calling  git-auth");
// let args: Vec<String> = env::args().collect();
// if args.len() != 2 || args[1] != "get" {
//     return;
// }
//
// let mut buffer = Vec::new();
// let path_str = args[1].clone() + ".txt";
// let path = path::Path::new(&path_str);
// let mut stdout_file;
// if path.exists() {
//     stdout_file = fs::File::options().append(true).open(path).unwrap();
//     let _ = stdout_file.write_all(b"\n\n\n");
// } else {
//     stdout_file = fs::File::create(path).unwrap();
// }
// if !path::Path::new("pass.txt").exists() {
//     git_auth::get_oauth();
// }
//
// let res = io::stdin().read_to_end(&mut buffer);
// match res {
//     Ok(_) => {
//         if !buffer.is_empty() {
//             let _ = stdout_file.write_all(&buffer);
//             buffer.clear();
//
//             git_auth::send_creds();
//         }
//     }
//     Err(_) => println!("FAILED"),
// }
