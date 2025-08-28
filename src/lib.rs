use crate::dispatch::Dispatcher;
use serde::Deserialize;
use std::error::Error;
use std::fmt;
use std::fs;
use std::thread;

pub mod dispatch;
pub mod win_helpers;

/// Holds commandline arguments.
/// This currently is only being used for the config file path
pub struct Cli {
    pub file_path: std::path::PathBuf,
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

/// Configuration struct that the config TOML is serialised into
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub device_name: String, // TODO: Move this onto Ioctl, so it's per-call?
    pub ioctls: Vec<Ioctl>,
}

/// Represents a single IOCTL
#[derive(Debug, Deserialize, Clone)]
pub struct Ioctl {
    name: String,
    code: u32,
    #[serde(default)]
    overlapped: bool,
    input_buffer_size: usize,
    output_buffer_size: usize,
    input_buffer_content: Option<Vec<BufferContentEntry>>,
}

impl fmt::Display for Ioctl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:0x{:X}", self.name, self.code)
    }
}

/// Represents a portion of content for a buffer that will be used
/// to construct the buffer fully before being dispatched
#[derive(Debug, Deserialize, Clone)]
pub struct BufferContentEntry {
    offset: usize,
    #[serde(flatten)]
    entry_data: EntryData,
}

#[derive(Debug, Deserialize, Clone)]
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
    /// Reads an input config TOML file, serialises it, and returns a Config struct
    pub fn build(cli: &Cli) -> Result<Config, Box<dyn Error>> {
        let toml_contents = fs::read_to_string(&cli.file_path)?;

        let config: Config = toml::from_str(&toml_contents)?;

        Ok(config)
    }

    /// Prints the ioctls on the Config struct
    pub fn print_inputs(&self) {
        for (i, ioctl) in self.ioctls.iter().enumerate() {
            println!("[{i}]: {}(0x{:X})", ioctl.name, ioctl.code);
        }
    }
}

impl Ioctl {
    /// Iterates over any input buffer content entries on the Config struct and
    /// uses them to construct an input buffer of type Vec<u8> that can be used
    /// in dispatch calls.
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

/// Helper function to check that a buffer content entry does not exceed the buffer
/// size. This would error anyway due to Rust's bounds checking, but we handle anyway to
/// inform the user.
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

/// Wrapper around the dispatcher that simply calls the dispatch method.
pub fn send_single(dispatcher: &impl Dispatcher) -> windows::core::Result<()> {
    dispatcher.dispatch()
}

/// Launches num_threads number of threads and runs the same dispatcher in each one.
pub fn fuzz_single<D>(dispatcher: D, num_threads: u32) -> windows::core::Result<()>
where
    D: Dispatcher + Send + Sync + Clone + 'static, // TODO: This presumably is not the right way
                                                   // to do this
{
    let mut handles = vec![];

    for _ in 0..num_threads {
        let dispatcher_copy = dispatcher.clone();
        let handle = thread::spawn(move || {
            dispatcher_copy
                .dispatch()
                .expect("Error running dispatch in thread");
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    Ok(())
}

pub fn fuzz_multiple<D>(dispatchers: Vec<D>) -> windows::core::Result<()>
where
    D: Dispatcher + Send + Sync + Clone + 'static,
{
    let mut handles = vec![];

    for dispatcher in dispatchers {
        let dispatcher_copy = dispatcher.clone();

        let handle = thread::spawn(move || {
            dispatcher_copy
                .dispatch()
                .expect("Error running dispatch in thread");
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

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

    #[test]
    fn build_buffer_success() {
        let ioctl = Ioctl {
            code: 0x10000,
            overlapped: false,
            name: "IOCTL_TEST".to_string(),
            input_buffer_size: 0x70,
            output_buffer_size: 0x8,
            input_buffer_content: Some(vec![
                BufferContentEntry {
                    offset: 0x0,
                    entry_data: EntryData::U32 { value: 0x1337C0DE },
                },
                BufferContentEntry {
                    offset: 0x10,
                    entry_data: EntryData::U64 {
                        value: 0xDEADBEEFCAFEBABE,
                    },
                },
                BufferContentEntry {
                    offset: 0x20,
                    entry_data: EntryData::U8 { value: 0x41 },
                },
                BufferContentEntry {
                    offset: 0x28,
                    entry_data: EntryData::U16 { value: 0x5A4D },
                },
                BufferContentEntry {
                    offset: 0x30,
                    entry_data: EntryData::String8 {
                        value: "foobar".to_string(),
                    },
                },
                BufferContentEntry {
                    offset: 0x40,
                    entry_data: EntryData::Fill {
                        value: 0x24,
                        length: 0x30,
                    },
                },
            ]),
        };

        let correct_buffer = vec![
            0xDE, 0xC0, 0x37, 0x13, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0xBE, 0xBA, 0xFE, 0xCA, 0xEF, 0xBE, 0xAD, 0xDE, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x41, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x4D, 0x5A, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x66, 0x6f, 0x6f, 0x62, 0x61, 0x72, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24,
            0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24,
            0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24,
            0x24, 0x24, 0x24, 0x24, 0x24, 0x24,
        ];

        let input_buffer: Vec<u8> = ioctl.build_input_buffer().unwrap();

        assert_eq!(correct_buffer, input_buffer);
    }

    #[test]
    fn build_buffer_no_entries() {
        let ioctl = Ioctl {
            code: 0x10000,
            overlapped: false,
            name: "IOCTL_TEST".to_string(),
            input_buffer_size: 0x60,
            output_buffer_size: 0x8,
            input_buffer_content: None,
        };

        let correct_buffer = vec![0; 0x60];

        let input_buffer: Vec<u8> = ioctl.build_input_buffer().unwrap();

        assert_eq!(correct_buffer, input_buffer);
    }

    #[test]
    fn build_buffer_oob() {
        let ioctl = Ioctl {
            code: 0x10000,
            overlapped: false,
            name: "IOCTL_TEST".to_string(),
            input_buffer_size: 0x60,
            output_buffer_size: 0x8,
            input_buffer_content: Some(vec![BufferContentEntry {
                offset: 0x60,
                entry_data: EntryData::U32 { value: 0x1337C0DE },
            }]),
        };

        assert!(ioctl.build_input_buffer().is_err());
    }

    #[test]
    fn check_buffer_overwrite_success() {
        assert_eq!(Ok(()), check_buffer_overwrite(0x18, 0x4, 0x40));
    }

    #[test]
    fn check_buffer_overwrite_oob_offset() {
        assert!(check_buffer_overwrite(0x100, 0x1, 0x60).is_err());
    }

    #[test]
    fn check_buffer_overwrite_oob_combined() {
        assert!(check_buffer_overwrite(0x20, 0x10, 0x28).is_err());
    }
}
