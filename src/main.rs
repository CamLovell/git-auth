use clap::{Parser, Subcommand};
use git_auth::{GitRequest, db, github, send_creds};

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
            let git_request = GitRequest::from_stdin();
            eprintln!("{:?}", git_request);
            let creds = github::get_login()?;
            let conn = db::open()?;
            db::add_login(&conn, &creds)?;
            send_creds(&creds);
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
