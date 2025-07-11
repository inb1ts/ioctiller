use ioctiller::{Cli, Config};
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    let cli = Cli::build(&args).unwrap_or_else(|err| {
        eprintln!("Problem parsing file path arg: {err}");
        process::exit(1);
    });

    let config = Config::build(&cli).unwrap_or_else(|err| {
        eprintln!("Problem loading config file contents: {err}");
        process::exit(1);
    });

    if let Err(e) = ioctiller::run(config) {
        eprintln!("Error running ioctiller: {e}");
        process::exit(1);
    }
}
