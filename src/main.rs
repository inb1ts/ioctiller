use inquire::Select;
use ioctiller::dispatch::IoctlDispatcher;
use ioctiller::{Cli, Config, Ioctl};
use std::env;
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
    let config_clone = config.clone();

    let selected_ioctl: Ioctl = Select::new("Please select the IOCTL to send", config_clone.ioctls)
        .prompt()
        .expect("Error selecting IOCTL");

    // Send selected IOCTL
    let ioctl_dispatcher = IoctlDispatcher {
        device_name: config.device_name,
        ioctl: &selected_ioctl,
    };

    if let Err(e) = ioctiller::send(&ioctl_dispatcher) {
        eprintln!("Error running ioctiller: {e}");
        process::exit(1);
    }
}
