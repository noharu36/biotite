use biotite::cli::{Cli, Commands};
use biotite::cli::build::build;
use biotite::cli::serve::start_server;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), std::io::Error>{
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Build { serve, directory, output }) => {
            build(directory, output)?;

            if *serve {
                start_server(output).await?;
            }

            Ok(())
        },
        None => {Ok(())}
    }
}
