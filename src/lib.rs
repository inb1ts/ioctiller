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
    pub fn build(cli: &Cli) -> Result<Config, Box<dyn Error>> {
        let toml_contents = fs::read_to_string(&cli.file_path)?;

        let config: Config = toml::from_str(&toml_contents)?;

        Ok(config)
    }
}

impl Ioctl {
    pub fn build_input_buffer(&self) -> Result<Vec<u8>, &'static str> {
        let mut buffer = vec![0; self.input_buffer_size as usize];

        let buffer_content_entries = match &self.input_buffer_content {
            Some(buffer_content_entries) => buffer_content_entries,
            None => return Ok(buffer),
        };

        for entry in buffer_content_entries {
            match &entry.entry_data {
                crate::EntryData::U8 { value } => {
                    check_buffer_overwrite(entry.offset, size_of::<u8>(), self.input_buffer_size)?;

                    buffer[entry.offset as usize] = *value;
                }
                crate::EntryData::U16 { value } => {
                    let u16_size = size_of::<u16>();
                    check_buffer_overwrite(entry.offset, u16_size, self.input_buffer_size)?;

                    buffer[entry.offset..entry.offset + u16_size]
                        .copy_from_slice(&(*value).to_le_bytes());
                }
                crate::EntryData::U32 { value } => {
                    let u32_size = size_of::<u32>();
                    check_buffer_overwrite(entry.offset, u32_size, self.input_buffer_size)?;

                    buffer[entry.offset..entry.offset + u32_size]
                        .copy_from_slice(&(*value).to_le_bytes());
                }
                crate::EntryData::U64 { value } => {
                    let u64_size = size_of::<u64>();
                    check_buffer_overwrite(entry.offset, u64_size, self.input_buffer_size)?;

                    buffer[entry.offset..entry.offset + u64_size]
                        .copy_from_slice(&(*value).to_le_bytes());
                }
                crate::EntryData::String8 { value } => {
                    let str_size = value.len();
                    check_buffer_overwrite(entry.offset, str_size, self.input_buffer_size)?;

                    buffer[entry.offset..entry.offset + str_size].copy_from_slice(value.as_bytes());
                }
                crate::EntryData::Fill { value, length } => {
                    check_buffer_overwrite(entry.offset, *length, self.input_buffer_size)?;

                    buffer[entry.offset..entry.offset + length].fill(*value);
                }
            }
        }

        Ok(buffer)
    }
}

fn check_buffer_overwrite(
    offset: usize,
    entry_size: usize,
    buffer_size: usize,
) -> Result<(), &'static str> {
    if (offset + entry_size) > buffer_size {
        return Err("Input buffer entry content is out of bounds");
    }
    Ok(())
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
}
