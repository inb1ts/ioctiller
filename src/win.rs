use super::Ioctl;
use std::error::Error;
use std::io;
use windows::{
    Win32::Foundation::*, Win32::Storage::FileSystem::*, Win32::System::IO::*, core::PCWSTR,
};
use windows_strings::HSTRING;

pub fn send_ioctl(device_name: &String, ioctl: &Ioctl) -> windows::core::Result<()> {
    println!("Sending {}", ioctl.name);

    let device_name_arg = HSTRING::from(device_name);
    let device_name_arg = PCWSTR::from_raw(device_name_arg.as_ptr());

    let device_handle = open_device_handle(device_name_arg)?;

    unsafe {
        windows::Win32::Foundation::CloseHandle(device_handle)?;
    }

    println!("Device handle closed successfully.");
    Ok(())
}

fn open_device_handle(device_name: PCWSTR) -> windows::core::Result<HANDLE> {
    unsafe {
        CreateFileW(
            device_name,
            GENERIC_READ.0 | GENERIC_WRITE.0 | DELETE.0,
            FILE_SHARE_NONE,
            None,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            None,
        )
    }
}
