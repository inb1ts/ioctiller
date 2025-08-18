use crate::Ioctl;
use crate::win_helpers::{open_device_handle, send_device_io_control};
use windows::Win32::Foundation::HANDLE;

const KERNEL_ADDR_MIN: u64 = 0xFFFF800000000000;

/// Describes a struct that can take some form of input and send it to a destination.
/// Current implementation will cover dispatchers for IOCTLs and Filter Communication Port
/// messages, but could also extend to other generic OS functionality such as spawning processes.
///
/// Also allows for mocking out calls in tests where we can't actually communicate with
/// a driver.
pub trait Dispatcher {
    fn dispatch(&self) -> windows::core::Result<()>;
}

/// Dispatcher used to call DeviceIoControl in order to send an IRP to a specified
/// IOCTL for a driver.
pub struct IoctlDispatcher<'a> {
    pub device_name: String,
    pub ioctl: &'a Ioctl,
}

impl<'a> Dispatcher for IoctlDispatcher<'a> {
    fn dispatch(&self) -> windows::core::Result<()> {
        println!("Sending {} to {}", self.ioctl.name, self.device_name);

        let device_handle: HANDLE = open_device_handle(&self.device_name)?;

        let output_buffer = send_device_io_control(device_handle, self.ioctl)?;

        println!("DeviceIoControl called successfully.");

        unsafe {
            windows::Win32::Foundation::CloseHandle(device_handle)?;
        }

        println!("Device handle closed successfully.");

        if output_buffer.len() > 0 {
            println!("Output:\n{:X?}\n", output_buffer);
        } else {
            println!("No output buffer received");
        }

        Ok(())
    }
}

// Dispatcher helpers

/// Iterates through a buffer in pointer-sized chunks, and checks to see whether
/// value falls within kernel address range. Returns a vec of (offset, leaked_addr) tuples
fn check_info_leaks(buffer: &Vec<u8>) -> Vec<(u64, u64)> {
    let mut found_addresses = Vec::new();

    let mut offset = 0;
    for chunk in buffer.chunks_exact(4) {
        let potential_addr = u64::from_ne_bytes(chunk.try_into().unwrap());

        if potential_addr >= KERNEL_ADDR_MIN {
            found_addresses.push((offset, potential_addr));
        }

        offset += 8;
    }

    found_addresses
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_info_leaks_success() {
        let test_buffer = vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0x78, 0x56, 0x34, 0x12, 0xFF, 0xFF, 0xFF, 0xFF, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x80, 0xFF, 0xFF, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0x10, 0x32, 0x54, 0x76, 0x98, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0, 0, 0, 0, 0x24, 0xbe, 0x6a, 0x52, 0xFF, 0xFF,
            0xFF, 0xFF,
        ];

        let test_output = check_info_leaks(&test_buffer);

        let correct_output: Vec<(u64, u64)> = vec![
            (8, 0xFFFFFFFF12345678),
            (64, 0xFFFF800000000000),
            (82, 0xFFFFFF9876543210),
            (90, 0xFFFFFFFFFFFFFFFF),
            (102, 0xFFFFFFFF526ABE24),
        ];

        assert_eq!(correct_output, test_output);
    }
}
