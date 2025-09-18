use clap::{Parser, Subcommand};
use colored::Colorize;
use git_auth::{
    Request, db,
    error::{DatabaseError, Error, GithubError},
    github, send_creds,
};
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
    /// (called by git) Retrieve a credential for a given host/protocol/path
    Get,
    /// (called by git) "Store" credential, for us mark as valid
    Store,
    /// (called by git) "Erase" credential, for us mark as invalid
    Erase,
    /// Delete credentials database
    Purge,
    /// Initialise git-auth, setting expected git config etc.
    Init,
}

fn main() -> Result<(), Error> {
    // Force colours
    colored::control::set_override(true);
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            init();
        }
        Commands::Purge => {
            purge();
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

fn init() {
    let exe_path = env::current_exe().expect("I mean, we are running??");
    let exe_path = String::from("!")
        + exe_path
            .to_str()
            .expect("exe path should be able to be a string");

    if let Err(err) = Command::new("git")
        .args(["config", "set", "--global", "credential.helper", &exe_path])
        .output()
    {
        eprintln!(
            "{} setting credential helper: {}",
            "Error".red().bold(),
            err
        );
        let config_cmd = format!("git config set --global credential.helper {}", exe_path);
        eprintln!(
            "\tTo set this manually either run:\n\n\t{}\n",
            config_cmd.cyan()
        );
        let config_item = format!("[credential]\n\thelper={}", exe_path);
        eprintln!("Or add this to your .gitconfig\n\t{}", config_item.cyan());
    }

    if let Err(err) = Command::new("git")
        .args([
            "config",
            "set",
            "--global",
            "credential.usehttppath",
            "true",
        ])
        .output()
    {
        eprintln!(
            "{} changing credential settings: {}",
            "Error".red().bold(),
            err
        );
        eprintln!(
            "\tTo set this manually either run:\n\n\t{}\n",
            "git config set --global credential.usehttppath true".cyan()
        );
        eprintln!(
            "Or add this to your .gitconfig\n\t{}",
            "[credential]\n\tusehttppath=true".cyan()
        );
    }

    if let Err(err) = Command::new("git")
        .args(["config", "--global", "alias.auth", &exe_path])
        .output()
    {
        eprintln!("{} setting git-auth alias: {}", "Error".red().bold(), err);
        let config_cmd = format!("git config --global alias.auth {}", exe_path);
        eprintln!(
            "\tTo set this manually either run:\n\n\t{}\n",
            config_cmd.cyan()
        );
        let config_item = format!("[alias]\n\tauth={}", exe_path);
        eprintln!("Or add this to your .gitconfig\n\t{}", config_item.cyan());
    }
}

fn purge() {
    let path = match env::home_dir() {
        Some(p) => p.join(".local/share/git-auth/creds.db"),
        None => {
            eprintln!("Error purging: {}", "home_dir not defined".red());
            return;
        }
    };

    if !path.exists() {
        eprintln!("No database, nothing to do");
        return;
    }

    if !Confirm::new(
        "This will erase ALL credentials for ALL remotes and repositories.\nAre you sure?",
    )
    .with_default(false)
    .prompt()
    .unwrap_or(false)
    {
        eprintln!("Purge canceled");
        return;
    }

    match db::open().and_then(|c| db::fetch_all_logins(&c).map_err(DatabaseError::from)) {
        Ok(logins) => logins.iter().for_each(|l| match l.delete_password() {
            Ok(_) => eprintln!("Deleted login for: {l}"),
            Err(err) => eprintln!("Failed to delete login for: {l}\nReason: {err}"),
        }),
        Err(err) => eprintln!("Error deleting passwords from keyring.\nReason: {err}"),
    }

    match fs::remove_file(path) {
        Ok(_) => eprintln!("Database deleted"),
        Err(err) => eprintln!("Error deleting databse: {err}"),
    }

    eprintln!("{}: purge complete!", "Success".green())
}

fn get() -> Result<(), Error> {
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
            let logins =
                db::fetch_available_logins(&conn, &git_request).map_err(DatabaseError::from)?;
            let creds = if logins.is_empty()
                || Confirm::new("No login found, create new?")
                    .with_default(true)
                    .prompt()?
            {
                github::get_login()?
            } else {
                Select::new("Select existing login:\n", logins)
                    .without_help_message()
                    .without_filtering()
                    .with_vim_mode(true)
                    .prompt()?
            };

            let user_id = db::add_login(&conn, &creds).map_err(DatabaseError::from)?;
            db::add_request(&conn, &git_request, &user_id).map_err(DatabaseError::from)?;
            creds
        }
    };

    send_creds(&creds).map_err(GithubError::from)?;
    Ok(())
}

fn store() -> Result<(), Error> {
    let git_request = Request::from_stdin()?;
    let conn = db::open()?;
    if let (login, false) = db::fetch_login(&conn, &git_request).map_err(DatabaseError::from)? {
        let disp_str = format!(
            "Storing valid credential {} for {}/{}",
            login.username, git_request.host, git_request.owner
        );
        eprintln!("{}", disp_str.green().bold());
        db::validate_request(&conn, &git_request, true).map_err(DatabaseError::from)?;
    }
    Ok(())
}

fn erase() -> Result<(), Error> {
    let git_request = Request::from_stdin()?;
    let conn = db::open()?;
    eprintln!(
        "Existing credential for {} found to be {}.\nRe-run your command to obtian a new login\n",
        git_request.host,
        "invalid".red().bold()
    );
    Ok(db::validate_request(&conn, &git_request, false).map_err(DatabaseError::from)?)
}
