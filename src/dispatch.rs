use crate::Ioctl;
use crate::win_helpers::{open_device_handle, send_device_io_control};
use windows::Win32::Foundation::HANDLE;

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

        let device_handle: HANDLE = open_device_handle(&self.device_name, self.ioctl.overlapped)?;

        let input_buffer = self.ioctl.build_input_buffer().unwrap();

        let output_buffer = send_device_io_control(
            device_handle,
            self.ioctl.code,
            input_buffer,
            self.ioctl.input_buffer_size,
            self.ioctl.output_buffer_size,
        )?;

        println!("DeviceIoControl called successfully.");

        unsafe {
            windows::Win32::Foundation::CloseHandle(device_handle)?;
        }

        println!("Device handle closed successfully.");

        if output_buffer.len() > 0 {
            println!("Output:\n{:X?}\n", output_buffer);

            if let Some(possible_info_leaks) = check_info_leaks(&output_buffer) {
                for leak in possible_info_leaks {
                    println!("Possible leak at {}: {}", leak.0, leak.1);
                }
            }
        } else {
            println!("No output buffer received");
        }

        Ok(())
    }
}

// Dispatcher helpers

/// Iterates through a buffer in pointer-sized chunks, and checks to see whether
/// value falls within kernel address range. Returns a vec of (offset, leaked_addr) tuples
/// This iterates through the buffer 2-bytes at a time, which is a crude way of increasing the
/// likelihood of catching things at weird offsets, but reducing some of the false positives from
/// a 1-byte sliding window
fn check_info_leaks(buffer: &Vec<u8>) -> Option<Vec<(usize, u64)>> {
    const KERNEL_ADDR_MIN: u64 = 0xFFFF800000000000;
    const POINTER_SIZE: usize = 8;

    let mut found_addresses = Vec::new();

    let mut i = 0;
    while i <= buffer.len().saturating_sub(POINTER_SIZE) {
        let curr_offset = &buffer[i..i + POINTER_SIZE];
        let potential_addr = u64::from_ne_bytes(curr_offset.try_into().unwrap());

        if potential_addr >= KERNEL_ADDR_MIN {
            found_addresses.push((i, potential_addr));
            i += POINTER_SIZE;
        } else {
            i += 2;
        }
    }

    if found_addresses.len() == 0 {
        return None;
    }

    Some(found_addresses)
}

#[cfg(test)]
mod tests {
    use core::panic;

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

        let correct_output: Vec<(usize, u64)> = vec![
            (8, 0xFFFFFFFF12345678),
            (64, 0xFFFF800000000000),
            (82, 0xFFFFFF9876543210),
            (90, 0xFFFFFFFFFFFFFFFF),
            (102, 0xFFFFFFFF526ABE24),
        ];

        if let Some(test_output) = check_info_leaks(&test_buffer) {
            assert_eq!(correct_output, test_output)
        } else {
            panic!("No info leaks found")
        }
    }
}
