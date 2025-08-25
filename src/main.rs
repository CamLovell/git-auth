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
            if Confirm::new("Existing login found but may be invalid. Try anyway?")
                .with_default(false)
                .prompt()?
            {
                login
            } else {
                github::get_login()?
            }
        }
        Err(_) => {
            let logins = db::fetch_available_logins(&conn, &git_request)?;
            let creds = if logins.is_empty()
                || Confirm::new("No login found, create new?")
                    .with_default(true)
                    .prompt()?
            {
                github::get_login()?
            } else {
                Select::new("select existing or create", logins)
                    .without_help_message()
                    .without_filtering()
                    .with_vim_mode(true)
                    .prompt()?
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
