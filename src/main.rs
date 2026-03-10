mod artifact;
mod cli;
mod commands;
mod error;
mod metadata;
mod storage;

use clap::Parser;
use cli::{Cli, Commands};

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init => {
            let cwd = std::env::current_dir().expect("cannot determine current directory");
            commands::init::run(&cwd)
        }
        Commands::Write {
            kind,
            content,
            based_on,
            tag,
            agent,
            task,
            name,
            format,
            json,
            no_forward,
        } => commands::write::run(&commands::write::WriteArgs {
            kind,
            content,
            based_on,
            tags: tag,
            agent,
            task,
            name,
            format,
            json,
            no_forward,
        }),
        Commands::Find {
            query,
            kind,
            tag,
            task,
            limit,
            json,
        } => commands::find::run(&commands::find::FindArgs {
            query,
            kind,
            tag,
            task,
            limit,
            json,
        }),
        Commands::Show {
            id,
            json,
            content_only,
        } => commands::show::run(&commands::show::ShowArgs {
            id,
            json,
            content_only,
        }),
        Commands::List {
            kind,
            tag,
            limit,
            json,
        } => commands::list::run(&commands::list::ListArgs {
            kind,
            tag,
            limit,
            json,
        }),
        Commands::Upstream {
            id,
            recursive,
            json,
        } => commands::upstream::run(&commands::upstream::UpstreamArgs {
            id,
            recursive,
            json,
        }),
        Commands::Downstream {
            id,
            recursive,
            json,
        } => commands::downstream::run(&commands::downstream::DownstreamArgs {
            id,
            recursive,
            json,
        }),
        Commands::Doctor => commands::doctor::run(),
    };

    if let Err(e) = result {
        eprintln!("stdai: {}", e);
        std::process::exit(1);
    }
}
