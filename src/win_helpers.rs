use super::Ioctl;
use windows::{
    Win32::Foundation::*, Win32::Storage::FileSystem::*, Win32::System::IO::DeviceIoControl,
    core::PCWSTR,
};
use windows_strings::HSTRING;

pub fn open_device_handle(device_name: &String) -> windows::core::Result<HANDLE> {
    let device_name_arg = HSTRING::from(device_name);
    let device_name_arg = PCWSTR::from_raw(device_name_arg.as_ptr());

    unsafe {
        CreateFileW(
            device_name_arg,
            GENERIC_READ.0 | GENERIC_WRITE.0, // This will need to be configurable.
            FILE_SHARE_NONE,
            None,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            None,
        )
    }
}

pub fn send_device_io_control(device_handle: HANDLE, ioctl: &Ioctl) -> windows::core::Result<()> {
    let mut bytes_returned: u32 = 0;
    // TODO: This needs to be decoupled
    let input_buffer = ioctl.build_input_buffer().unwrap(); // TODO: Handle this properly.
    let output_buffer = vec![0; ioctl.output_buffer_size];

    println!("Sending input buffer...");

    unsafe {
        DeviceIoControl(
            device_handle,
            ioctl.code,
            Some(input_buffer.as_ptr() as *const _),
            ioctl.input_buffer_size.try_into()?,
            Some(output_buffer.as_ptr() as *mut _),
            ioctl.output_buffer_size.try_into()?,
            Some(&mut bytes_returned),
            None,
        )?;
    }

    if output_buffer.len() > 0 {
        if bytes_returned == 0 {
            println!("No output received.");
        }

        println!("Output:\n{:X?}\n", output_buffer);
    }

    Ok(())
}
