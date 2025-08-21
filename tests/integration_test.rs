use ioctiller::dispatch::Dispatcher;
use ioctiller::{Cli, Config, Ioctl};

pub struct TestDispatcher<'a> {
    pub device_name: String,
    pub ioctl: &'a Ioctl,
}

impl<'a> Dispatcher for TestDispatcher<'a> {
    fn dispatch(&self) -> windows::core::Result<()> {
        let input = self.ioctl.build_input_buffer().unwrap();

        assert_eq!(
            self.device_name,
            "\\\\.\\GLOBALROOT\\Device\\Beep".to_string()
        );

        assert_eq!(input.len(), 0x40);

        let correct_buffer = vec![
            0x41, 0, 0, 0, 0, 0, 0, 0, 0x34, 0x12, 0, 0, 0, 0, 0, 0, 0xDE, 0xC0, 0xAD, 0xDE, 0, 0,
            0, 0, 0xEF, 0xBE, 0xAD, 0xDE, 0xBE, 0xBA, 0xFE, 0xCA, 0x46, 0x4F, 0x4F, 0x42, 0x41,
            0x52, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24,
            0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24,
        ];

        assert_eq!(input, correct_buffer);

        Ok(())
    }
}

#[test]
fn load_config() {
    let conf_path = std::path::PathBuf::from(r"tests\test.toml");
    let cli = Cli {
        file_path: conf_path,
    };

    let config = Config::build(&cli).unwrap();

    let selected_ioctl: &Ioctl = &config.ioctls[0];

    let test_dispatcher = TestDispatcher {
        device_name: config.device_name,
        ioctl: selected_ioctl,
    };

    if let Err(e) = ioctiller::send_single(&test_dispatcher) {
        panic!("Error calling send with test_dispatcher: {e}");
    }
}
