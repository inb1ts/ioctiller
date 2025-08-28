use inquire::{MultiSelect, Select, list_option::ListOption, prompt_u32, validator::Validation};
use ioctiller::dispatch::{FuzzIoctlDispatcher, SingleIoctlDispatcher};
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

    // Prompt user for mode
    let mode_options: Vec<&str> = vec!["Send single", "Fuzz single", "Fuzz multiple"];
    let mode: &str = Select::new("What would you like to do?", mode_options)
        .prompt()
        .expect("Error selecting mode");

    match mode {
        "Send single" => {
            // Inquire's Select option requires that the option vec is moved. Therefore we clone it,
            // and then the selected IOCTL is just returned from the clone
            let config_clone = config.clone();
            let selected_ioctl: Ioctl =
                Select::new("Please select the IOCTL to send", config_clone.ioctls)
                    .prompt()
                    .expect("Error selecting IOCTL");

            let ioctl_dispatcher = SingleIoctlDispatcher {
                device_name: config.device_name,
                ioctl: &selected_ioctl,
            };

            if let Err(e) = ioctiller::send_single(&ioctl_dispatcher) {
                eprintln!("Error running ioctiller: {e}");
                process::exit(1);
            }
        }
        "Fuzz single" => {
            let config_clone = config.clone();
            let selected_ioctl: Ioctl =
                Select::new("Please select the IOCTL to fuzz", config_clone.ioctls)
                    .prompt()
                    .expect("Error selecting IOCTL");

            let num_threads = prompt_u32("Number of threads:").expect("Invalid number of threads");

            if num_threads == 0 {
                eprintln!("Number of threads cannot be 0");
                process::exit(1);
            }

            let ioctl_dispatcher = FuzzIoctlDispatcher {
                device_name: config_clone.device_name,
                ioctl: selected_ioctl,
            };

            if let Err(e) = ioctiller::fuzz_single(ioctl_dispatcher, num_threads) {
                eprintln!("Error running ioctiller: {e}");
                process::exit(1);
            }
        }
        "Fuzz multiple" => {
            let config_clone = config.clone();

            let validator = |a: &[ListOption<&Ioctl>]| {
                if a.len() < 2 {
                    return Ok(Validation::Invalid("This list is too small".into()));
                }

                Ok(Validation::Valid)
            };

            let ans = MultiSelect::new(
                "Select the IOCTLS you would like to fuzz together:",
                config_clone.ioctls,
            )
            .with_validator(validator)
            .prompt();

            match ans {
                Ok(ioctls) => {
                    // TODO: This currently won't generic input types (when we need to account for
                    // FilterCommPort messages
                    let mut dispatchers: Vec<FuzzIoctlDispatcher> = vec![];

                    for ioctl in ioctls {
                        dispatchers.push(FuzzIoctlDispatcher {
                            device_name: config_clone.device_name.clone(),
                            ioctl: ioctl,
                        })
                    }

                    if let Err(e) = ioctiller::fuzz_multiple(dispatchers) {
                        eprintln!("Error running fuzz multiple: {e}");
                        process::exit(1);
                    }
                }
                Err(_) => {
                    eprintln!("Error processing multiple fuzz options");
                    process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("Did not recognise mode option: {mode}");
            process::exit(1);
        }
    }
}
