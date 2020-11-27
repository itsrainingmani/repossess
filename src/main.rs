use repossess::{filehandle, Cli};
use std::error::Error;
use structopt::StructOpt;

fn main() -> Result<(), Box<dyn Error>> {
    let cli: Cli = Cli::from_args();

    let repo = filehandle::extract_repo_from_cli(&cli)?;

    Ok(())
}
