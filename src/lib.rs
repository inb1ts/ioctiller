use serde::Deserialize;
use std::error::Error;
use std::fs;
use std::io;

pub mod win;

pub struct Cli {
    file_path: std::path::PathBuf,
}

impl Cli {
    pub fn build(args: &[String]) -> Result<Cli, &'static str> {
        if args.len() != 2 {
            return Err("incorrect number of arguments provided");
        }

        let file_path = std::path::PathBuf::from(&args[1]);

        Ok(Cli { file_path })
    }
}

#[derive(Debug, Deserialize)]
pub struct Config {
    device_name: String,
    ioctls: Vec<Ioctl>,
}

#[derive(Debug, Deserialize)]
pub struct Ioctl {
    name: String,
    code: u32,
    input_buffer_size: usize,
    output_buffer_size: usize,
    input_buffer_content: Option<Vec<BufferContentEntry>>,
}

#[derive(Debug, Deserialize)]
pub struct BufferContentEntry {
    offset: usize,
    #[serde(flatten)]
    entry_data: EntryData,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum EntryData {
    U8 { value: u8 },
    U16 { value: u16 },
    U32 { value: u32 },
    U64 { value: u64 },
    String8 { value: String },
    Fill { value: u8, length: usize },
}

impl Config {
    // TODO: is it bad pratice to have two possible error types returned here?
    pub fn build(cli: &Cli) -> Result<Config, Box<dyn Error>> {
        let toml_contents = fs::read_to_string(&cli.file_path)?;

        let config: Config = toml::from_str(&toml_contents)?;

        Ok(config)
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    println!("Please select the IOCTL to send:\n");
    let ioctls_count = config.ioctls.len();

    for (i, ioctl) in config.ioctls.iter().enumerate() {
        println!("[{i}]: {}(0x{:X})", ioctl.name, ioctl.code);
    }
    println!("\nInput: ");

    let mut input = String::new();

    io::stdin().read_line(&mut input)?;

    // TODO: Can I use match here?
    let input: usize = input.trim().parse()?;
    if input > ioctls_count - 1 {
        return Err("input provided was not valid index".into());
    }

    let selected_ioctl: &Ioctl = &config.ioctls[input];

    win::send_ioctl(&config.device_name, selected_ioctl)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_build_correct_cmdline_args() {
        let args: Vec<String> = vec!["ioctiller.exe".to_string(), "C:\\test.toml".to_string()];

        let cli = Cli::build(&args).unwrap();
        let correct_path = std::path::PathBuf::from("C:\\test.toml");

        assert_eq!(correct_path, cli.file_path);
    }

    #[test]
    fn cli_build_no_cmdline_args() {
        let args: Vec<String> = vec!["ioctiller.exe".to_string()];

        assert!(Cli::build(&args).is_err());
    }

    #[test]
    fn cli_build_too_many_cmdline_args() {
        let args: Vec<String> = vec![
            "iotctiller.exe".to_string(),
            "C:\\test.toml".to_string(),
            "foobar".to_string(),
        ];

        assert!(Cli::build(&args).is_err());
    }

    // TODO: Remove
    // #[test]
    // fn config_load_valid_file() {
    //     let mut config = Config {
    //         file_contents: "\
    //         [device]
    //         name = \"\\\\.\\TestDriver\"
    //
    //         [[ioctl]]
    //         name = \"IOCTL_1\"
    //         code = 0x220004
    //         input_buffer_size = 64
    //         output_buffer_size = 128
    //
    //         [[iotctl]]
    //         name = \"IOCTL_2\"
    //         code = 0x220008
    //         input_buffer_size = 32
    //         output_buffer_size = 64
    //
    //         [[ioctl]]
    //         name = \"IOCTL_3\"
    //         code = 0x22000C
    //         input_buffer_size = 0
    //         output_buffer_size = 256"
    //             .to_string(),
    //     };
    //
    //     config.load()
    // }
}
