use anyhow::Result;
use clap::{Parser, Subcommand};
use proto::api::function_runner_service_client::FunctionRunnerServiceClient;

mod run;

#[derive(Subcommand, Debug)]
pub enum Command {
    Trigger {
        name: String,
    },
    Run {
        path: String
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

pub async fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Trigger { name } => {
            let mut client = FunctionRunnerServiceClient::connect("http://[::1]:54036").await?;

            let request = tonic::Request::new(proto::api::InvokeRequest {
                payload: Some("{\"name\":\"".to_string() + &name + "\"}")
            });

            let response = client.invoke(request).await?;

            println!("{:?}", response.into_inner().output);

            Ok(())
        }
        Command::Run { path } => {
            run::run(path).await?;
            Ok(())
        }
    }
}
