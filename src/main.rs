use anyhow::Context;
use clap::{Parser, Subcommand};
use colored::Colorize;
use git_auth::{Request, db, github, send_creds};
use inquire::{Confirm, Select};
use std::{env, fs, process::Command};

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
    Purge,
    Init,
}

fn main() -> anyhow::Result<()> {
    // Force colours
    colored::control::set_override(true);
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            init()?;
        }
        Commands::Purge => {
            purge()?;
        }
        Commands::Get => {
            get()?;
        }
        Commands::Store => {
            store()?;
        }
        Commands::Erase => {
            erase()?;
        }
    };
    Ok(())
}

fn init() -> anyhow::Result<()> {
    let exe_path = env::current_exe().expect("I mean, we are running??");
    let exe_path = String::from("!") + exe_path.to_str().context("Failed to get exe as sting")?;

    Command::new("git")
        .args(["config", "set", "--global", "credential.helper", &exe_path])
        .output()?;

    Command::new("git")
        .args([
            "config",
            "set",
            "--global",
            "credential.usehttppath",
            "true",
        ])
        .output()?;

    Command::new("git")
        .args(["config", "--global", "alias.auth", &exe_path])
        .output()?;

    Ok(())
}

fn purge() -> anyhow::Result<()> {
    let path = env::home_dir()
        .context("home is unknown")?
        .join(".local/share/git-auth/creds.db");

    if path.exists() {
        fs::remove_file(path)?;
    } else {
        eprintln!("No database, nothing to do");
    }
    Ok(())
}

fn get() -> anyhow::Result<()> {
    let git_request = Request::from_stdin()?;
    let conn = db::open()?;
    let creds = match db::fetch_login(&conn, &git_request) {
        Ok((login, true)) => login,
        Ok((login, false)) => {
            if Confirm::new("Existing login found but may be invalid. Try anyway? [y/n]")
                .prompt()
                .expect("should not fail")
            {
                login
            } else {
                github::get_login()?
            }
        }
        Err(_) => {
            let logins = db::fetch_available_logins(&conn, &git_request)?;
            let options: Vec<_> = logins
                .iter()
                .map(|l| match &l.email {
                    Some(email) => format!("{} ({})", l.username, email),
                    None => l.username.to_string(),
                })
                .collect();

            let creds = if logins.is_empty()
                || Confirm::new("No login found, create new?")
                    .prompt()
                    .expect("should not fail")
            {
                github::get_login()?
            } else {
                let ans = Select::new("select existing or create", options.clone())
                    .with_vim_mode(true)
                    .prompt()?;
                let idx = options
                    .iter()
                    .position(|o| o == &ans)
                    .expect("selected option must exist");
                logins[idx].clone()
            };

            let user_id = db::add_login(&conn, &creds)?;
            db::add_request(&conn, &git_request, &user_id)?;
            creds
        }
    };

    send_creds(&creds)
}

fn store() -> anyhow::Result<()> {
    let git_request = Request::from_stdin()?;
    let conn = db::open()?;
    if let (login, false) = db::fetch_login(&conn, &git_request)? {
        let disp_str = format!(
            "Storing valid credential {} for {}/{}",
            login.username,
            git_request.host,
            git_request.path.as_deref().unwrap_or("")
        );
        eprintln!("{}", disp_str.green().bold());
        db::validate_request(&conn, &git_request, true)?;
    }
    Ok(())
}

fn erase() -> anyhow::Result<usize> {
    let git_request = Request::from_stdin()?;
    let conn = db::open()?;
    let disp_str = format!("Invalidating existing credential for {}", git_request.host);
    eprintln!("{}", disp_str.red().bold());
    Ok(db::validate_request(&conn, &git_request, false)?)
}
