use clap::{Parser, Subcommand};

#[derive(Subcommand, Debug)]
pub enum Command {
    Trigger {
        name: String,
    },
}

#[derive(Parser, Debug)]
struct Commands {
    #[command(subcommand)]
    command: Command,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Trigger { name } => {
            println!("{}", name)
        }
    }
}
