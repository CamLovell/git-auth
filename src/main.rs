use clap::{Parser, Subcommand};
use git_auth::{Request, db, github, send_creds};

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
    let cli = Cli::parse();
    match cli.command {
        Commands::Get => {
            let git_request = Request::from_stdin()?;
            let conn = db::open()?;
            let creds = match db::fetch_login(&conn, &git_request) {
                Ok(login) => login,
                Err(_) => {
                    let creds = github::get_login()?;
                    let user_id = db::add_login(&conn, &creds)?;
                    db::add_request(&conn, &git_request, &user_id)?;
                    creds
                }
            };

            send_creds(&creds)?;
        }
        Commands::Init => eprintln!("Initialising"),
        Commands::Store => eprintln!("Storeing"),
        Commands::Erase => eprintln!("Eraseing"),
        Commands::Test => {
            let _conn = db::open()?;
        }
    };
    Ok(())
}
