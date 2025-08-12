use ioctiller::{Cli, Config};

#[test]
fn load_config() {
    let conf_path = std::path::PathBuf::from(r"tests\test.toml");
    let cli = Cli {
        file_path: conf_path,
    };

    let config = Config::build(&cli).unwrap();
}
