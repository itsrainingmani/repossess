use repossess::{Cli, filehandle};
use structopt::StructOpt;

fn main() {
    let cli = Cli::from_args();

    filehandle::download_file(&cli.url).unwrap_or_else(|err| {
        eprintln!("Error: {}", err);
    });
}
