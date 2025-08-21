use windows::{
    Win32::Foundation::*, Win32::Storage::FileSystem::*, Win32::System::IO::*,
    Win32::System::Threading::CreateEventW, core::PCWSTR,
};
use windows_strings::HSTRING;

pub fn open_device_handle(device_name: &String, overlapped: bool) -> windows::core::Result<HANDLE> {
    let device_name_arg = HSTRING::from(device_name);
    let device_name_arg = PCWSTR::from_raw(device_name_arg.as_ptr());

    let file_attributes: FILE_FLAGS_AND_ATTRIBUTES = match overlapped {
        false => FILE_ATTRIBUTE_NORMAL,
        true => FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OVERLAPPED,
    };

    unsafe {
        CreateFileW(
            device_name_arg,
            GENERIC_READ.0 | GENERIC_WRITE.0, // This will need to be configurable.
            FILE_SHARE_NONE,
            None,
            OPEN_EXISTING,
            file_attributes,
            None,
        )
    }
}

pub fn send_device_io_control(
    device_handle: HANDLE,
    ioctl_code: u32,
    input_buffer: Vec<u8>,
    input_buffer_size: usize,
    output_buffer_size: usize,
) -> windows::core::Result<Vec<u8>> {
    let mut bytes_returned: u32 = 0;
    let output_buffer = vec![0; output_buffer_size];

    unsafe {
        DeviceIoControl(
            device_handle,
            ioctl_code,
            Some(input_buffer.as_ptr() as *const _),
            input_buffer_size.try_into()?,
            Some(output_buffer.as_ptr() as *mut _),
            output_buffer_size.try_into()?,
            Some(&mut bytes_returned),
            None,
        )?;
    }

    Ok(output_buffer)
}

pub fn send_device_io_control_overlapped(
    device_handle: HANDLE,
    ioctl_code: u32,
    input_buffer: Vec<u8>,
    input_buffer_size: usize,
    output_buffer_size: usize,
    wait_overlapped: bool,
) -> windows::core::Result<Vec<u8>> {
    let mut bytes_returned: u32 = 0;
    let output_buffer = vec![0; output_buffer_size];

    unsafe {
        let mut overlapped_obj = OVERLAPPED {
            Anonymous: OVERLAPPED_0 {
                Anonymous: OVERLAPPED_0_0 {
                    Offset: 0,
                    OffsetHigh: 0,
                },
            },
            hEvent: CreateEventW(None, true, false, None)?,
            Internal: 0,
            InternalHigh: 0,
        };

        DeviceIoControl(
            device_handle,
            ioctl_code,
            Some(input_buffer.as_ptr() as *const _),
            input_buffer_size.try_into()?,
            Some(output_buffer.as_ptr() as *mut _),
            output_buffer_size.try_into()?,
            Some(&mut bytes_returned),
            Some(&mut overlapped_obj),
        )?;
    }

    Ok(output_buffer)
}
