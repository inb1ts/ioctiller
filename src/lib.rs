use std::error::Error;
use std::fs;

pub struct Config {
    file_path: std::path::PathBuf,
}

impl Config {
    pub fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() != 2 {
            return Err("incorrect number of arguments provided");
        }

        let file_path = std::path::PathBuf::from(&args[1]);

        Ok(Config { file_path })
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    println!("running");

    let contents = fs::read_to_string(config.file_path)?;

    println!("With text: \n{contents}");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_cmdline_args() -> Result<(), &'static str> {
        let args: Vec<String> = vec!["ioctiller.exe".to_string(), "C:\\test.toml".to_string()];

        let config = Config::build(&args).unwrap();
        let correct_path = std::path::PathBuf::from("C:\\test.toml");

        assert_eq!(correct_path, config.file_path);

        Ok(())
    }

    #[test]
    fn no_cmdline_args() -> Result<(), &'static str> {
        let args: Vec<String> = vec!["ioctiller.exe".to_string()];

        assert!(Config::build(&args).is_err());
        Ok(())
    }

    #[test]
    fn too_many_cmdline_args() -> Result<(), &'static str> {
        let args: Vec<String> = vec![
            "iotctiller.exe".to_string(),
            "C:\\test.toml".to_string(),
            "foobar".to_string(),
        ];

        assert!(Config::build(&args).is_err());
        Ok(())
    }
}
