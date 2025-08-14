use crate::Ioctl;
use crate::win_helpers::send_ioctl;

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
        send_ioctl(&self.device_name, &self.ioctl)
    }
}
