use ioctiller::dispatch::IoctlDispatcher;
use ioctiller::{Cli, Config, Ioctl};
use std::env;
use std::io;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Retrieve command-line args for config filepath
    let cli = Cli::build(&args).unwrap_or_else(|err| {
        eprintln!("Error parsing file path arg: {err}");
        process::exit(1);
    });

    // Parse config file
    let config = Config::build(&cli).unwrap_or_else(|err| {
        eprintln!("Error loading config file contents: {err}");
        process::exit(1);
    });

    // Prompt user to select IOCTL to send
    // TODO: Add interactive CLI when there are multiple options
    println!("Please select the IOCTL to send:\n");
    config.print_inputs();
    println!("\nInput: ");

    let mut input = String::new();

    io::stdin().read_line(&mut input).unwrap_or_else(|err| {
        eprintln!("Error reading input: {err}");
        process::exit(1);
    });

    let input: usize = input.trim().parse().unwrap_or_else(|err| {
        eprintln!("Error parsing input: {err}");
        process::exit(1);
    });

    if input > config.ioctls.len() - 1 {
        eprint!("input provided was not valid index");
        process::exit(1);
    }

    let selected_ioctl: &Ioctl = &config.ioctls[input];

    // Send selected IOCTL
    let ioctl_dispatcher = IoctlDispatcher {
        device_name: config.device_name,
        ioctl: selected_ioctl,
    };

    if let Err(e) = ioctiller::send(&ioctl_dispatcher) {
        eprintln!("Error running ioctiller: {e}");
        process::exit(1);
    }
}
