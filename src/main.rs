use clap::{Parser, Subcommand};
use colored::Colorize;
use git_auth::{Request, db, github, send_creds};
use inquire::{Confirm, Select};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    // Get a credential
    Test,
    Get,
    Store,
    Erase,
    Init,
}

fn main() -> anyhow::Result<()> {
    // Force colours
    colored::control::set_override(true);
    let cli = Cli::parse();
    let git_request = Request::from_stdin()?;
    let conn = db::open()?;

    match cli.command {
        Commands::Get => {
            let creds = match db::fetch_login(&conn, &git_request) {
                Ok(login) => login,
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

            send_creds(&creds)?;
        }
        Commands::Init => eprintln!("Initialising"),
        Commands::Store => {
            let disp_str = format!("Storing valid credential for {}", git_request.host);
            eprintln!("{}", disp_str.green().bold());
            db::validate_request(&conn, &git_request, true)?;
        }
        Commands::Erase => {
            let disp_str = format!("Invalidating existing credential for {}", git_request.host);
            eprintln!("{}", disp_str.red().bold());
            db::validate_request(&conn, &git_request, false)?;
        }
        Commands::Test => {
            let _conn = db::open()?;
        }
    };
    Ok(())
}
