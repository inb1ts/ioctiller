use super::Ioctl;
use windows::{
    Win32::Foundation::*, Win32::Storage::FileSystem::*, Win32::System::IO::DeviceIoControl,
    core::PCWSTR,
};
use windows_strings::HSTRING;

pub fn send_ioctl(device_name: &String, ioctl: &Ioctl) -> windows::core::Result<()> {
    println!("Sending {} to {}", ioctl.name, device_name);

    let device_name_arg = HSTRING::from(device_name);
    let device_name_arg = PCWSTR::from_raw(device_name_arg.as_ptr());

    let device_handle = open_device_handle(device_name_arg)?;

    send_device_io_control(device_handle, ioctl)?;

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

fn send_device_io_control(device_handle: HANDLE, ioctl: &Ioctl) -> windows::core::Result<()> {
    // TODO: Handle configuring buffers from config
    let mut bytes_returned: u32 = 0;
    let input_buffer: [u8; 0] = [];
    let output_buffer: [u8; 0] = [];

    unsafe {
        DeviceIoControl(
            device_handle,
            ioctl.code,
            Some(input_buffer.as_ptr() as *const _),
            ioctl.input_buffer_size,
            Some(output_buffer.as_ptr() as *mut _),
            ioctl.output_buffer_size,
            Some(&mut bytes_returned),
            None,
        )
    }
}

fn build_input_buffer(ioctl: &Ioctl) -> Result<Vec<u8>, &'static str> {
    let buffer = vec![0; ioctl.input_buffer_size];

    // TODO: Check there are input_buffer_contents
    let buffer_content_entries = match &ioctl.input_buffer_content {
        Some(buffer_content_entries) => buffer_content_entries,
        None => return Ok(buffer),
    };

    for entry in buffer_content_entries {
        match entry.entry_data {
            crate::EntryData::U8 => _,
            crate::EntryData::U16 _,
            crate::EntryData::U32 _,
            crate::EntryData::U64 _,
            crate::EntryData::String8 => _,
            crate::EntryData::Fill => _,
        }
    }

    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BufferContentEntry, EntryData};

    #[test]
    fn build_buffer_success() {
        let ioctl = Ioctl {
            code: 0x10000,
            name: "IOCTL_TEST".to_string(),
            input_buffer_size: 0x70,
            output_buffer_size: 0x8,
            input_buffer_content: Some(vec![
                BufferContentEntry {
                    offset: 0x0,
                    entry_data: EntryData::U32 { value: 0xC0DE },
                },
                BufferContentEntry {
                    offset: 0x10,
                    entry_data: EntryData::U64 { value: 0xDEADBEEF },
                },
                BufferContentEntry {
                    offset: 0x20,
                    entry_data: EntryData::U8 { value: 0x41 },
                },
                BufferContentEntry {
                    offset: 0x28,
                    entry_data: EntryData::U16 { value: 0x5A4D },
                },
                BufferContentEntry {
                    offset: 0x30,
                    entry_data: EntryData::String8 {
                        value: "foobar".to_string(),
                    },
                },
                BufferContentEntry {
                    offset: 0x40,
                    entry_data: EntryData::Fill {
                        value: 0x24,
                        length: 0x30,
                    },
                },
            ]),
        };

        let correct_buffer = vec![];

        let input_buffer: Vec<u8> = build_input_buffer(&ioctl);

        assert_eq!(correct_buffer, input_buffer);
    }
}
