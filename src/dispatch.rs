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

        let device_handle: HANDLE = open_device_handle(&self.device_name)?;

        send_device_io_control(device_handle, self.ioctl)?;

        println!("DeviceIoControl called successfully.");

        unsafe {
            windows::Win32::Foundation::CloseHandle(device_handle)?;
        }

        println!("Device handle closed successfully.");
        Ok(())
    }
}
