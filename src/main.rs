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

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Get => {
            let git_request = GitRequest::from_stdin();
            eprintln!("{:?}", git_request);
            let creds = get_oauth()?;
            send_creds(&creds);
        }
        Commands::Init => eprintln!("Initialising"),
        Commands::Store => eprintln!("Storeing"),
        Commands::Erase => eprintln!("Eraseing"),
    };
    Ok(())
}
