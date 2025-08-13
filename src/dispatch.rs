use crate::Ioctl;
use crate::win_helpers::send_ioctl;

pub trait Dispatcher {
    fn dispatch(&self) -> windows::core::Result<()>;
}

pub struct IoctlDispatcher<'a> {
    pub device_name: String,
    pub ioctl: &'a Ioctl,
}

impl<'a> Dispatcher for IoctlDispatcher<'a> {
    fn dispatch(&self) -> windows::core::Result<()> {
        send_ioctl(&self.device_name, &self.ioctl)
    }
}
