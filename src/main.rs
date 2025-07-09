use ioctiller::Config;
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    let config = Config::build(&args).unwrap_or_else(|err| {
        eprintln!("Problem parsing config file path: {err}");
        process::exit(1);
    });

    if let Err(e) = ioctiller::run(config) {
        eprintln!("Error running ioctiller: {e}");
        process::exit(1);
    }
}
