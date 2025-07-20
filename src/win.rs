use super::Ioctl;
use windows::{
    Win32::Foundation::*, Win32::Storage::FileSystem::*, Win32::System::IO::DeviceIoControl,
    core::PCWSTR,
};
use windows_strings::HSTRING;

pub fn send_ioctl(device_name: &String, ioctl: &Ioctl) -> windows::core::Result<()> {
    println!("Sending {} to {}", ioctl.name, device_name);

    let device_handle: HANDLE;
    let device_name_arg = HSTRING::from(device_name);
    let device_name_arg = PCWSTR::from_raw(device_name_arg.as_ptr());

    unsafe {
        device_handle = CreateFileW(
            device_name_arg,
            GENERIC_READ.0 | GENERIC_WRITE.0, // This will need to be configurable.
            FILE_SHARE_NONE,
            None,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            None,
        )?;
    }

    send_device_io_control(device_handle, ioctl)?;

    println!("DeviceIoControl called successfully.");

    unsafe {
        windows::Win32::Foundation::CloseHandle(device_handle)?;
    }

    println!("Device handle closed successfully.");
    Ok(())
}

fn send_device_io_control(device_handle: HANDLE, ioctl: &Ioctl) -> windows::core::Result<()> {
    let mut bytes_returned: u32 = 0;
    let input_buffer = build_input_buffer(ioctl).unwrap(); // TODO: Handle this properly.
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

fn check_buffer_overwrite(
    offset: usize,
    entry_size: usize,
    buffer_size: usize,
) -> Result<(), &'static str> {
    if (offset + entry_size) > buffer_size {
        return Err("Input buffer entry content is out of bounds");
    }
    Ok(())
}

fn build_input_buffer(ioctl: &Ioctl) -> Result<Vec<u8>, &'static str> {
    let mut buffer = vec![0; ioctl.input_buffer_size as usize];

    let buffer_content_entries = match &ioctl.input_buffer_content {
        Some(buffer_content_entries) => buffer_content_entries,
        None => return Ok(buffer),
    };

    for entry in buffer_content_entries {
        match &entry.entry_data {
            crate::EntryData::U8 { value } => {
                check_buffer_overwrite(entry.offset, size_of::<u8>(), ioctl.input_buffer_size)?;

                buffer[entry.offset as usize] = *value;
            }
            crate::EntryData::U16 { value } => {
                let u16_size = size_of::<u16>();
                check_buffer_overwrite(entry.offset, u16_size, ioctl.input_buffer_size)?;

                // TODO: This looks awful. There must be a simpler way?
                buffer[entry.offset..entry.offset + u16_size]
                    .copy_from_slice(&(*value).to_le_bytes());
            }
            crate::EntryData::U32 { value } => {
                let u32_size = size_of::<u32>();
                check_buffer_overwrite(entry.offset, u32_size, ioctl.input_buffer_size)?;

                buffer[entry.offset..entry.offset + u32_size]
                    .copy_from_slice(&(*value).to_le_bytes());
            }
            crate::EntryData::U64 { value } => {
                let u64_size = size_of::<u64>();
                check_buffer_overwrite(entry.offset, u64_size, ioctl.input_buffer_size)?;

                buffer[entry.offset..entry.offset + u64_size]
                    .copy_from_slice(&(*value).to_le_bytes());
            }
            crate::EntryData::String8 { value } => {
                let str_size = value.len();
                check_buffer_overwrite(entry.offset, str_size, ioctl.input_buffer_size)?;

                buffer[entry.offset..entry.offset + str_size].copy_from_slice(value.as_bytes());
            }
            crate::EntryData::Fill { value, length } => {
                check_buffer_overwrite(entry.offset, *length, ioctl.input_buffer_size)?;

                buffer[entry.offset..entry.offset + length].fill(*value);
            }
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
                    entry_data: EntryData::U32 { value: 0x1337C0DE },
                },
                BufferContentEntry {
                    offset: 0x10,
                    entry_data: EntryData::U64 {
                        value: 0xDEADBEEFCAFEBABE,
                    },
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

        let correct_buffer = vec![
            0xDE, 0xC0, 0x37, 0x13, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0xBE, 0xBA, 0xFE, 0xCA, 0xEF, 0xBE, 0xAD, 0xDE, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x41, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x4D, 0x5A, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x66, 0x6f, 0x6f, 0x62, 0x61, 0x72, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24,
            0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24,
            0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24,
            0x24, 0x24, 0x24, 0x24, 0x24, 0x24,
        ];

        let input_buffer: Vec<u8> = build_input_buffer(&ioctl).unwrap();

        assert_eq!(correct_buffer, input_buffer);
    }

    #[test]
    fn build_buffer_no_entries() {
        let ioctl = Ioctl {
            code: 0x10000,
            name: "IOCTL_TEST".to_string(),
            input_buffer_size: 0x60,
            output_buffer_size: 0x8,
            input_buffer_content: None,
        };

        let correct_buffer = vec![0; 0x60];

        let input_buffer: Vec<u8> = build_input_buffer(&ioctl).unwrap();

        assert_eq!(correct_buffer, input_buffer);
    }

    #[test]
    fn check_buffer_overwrite_success() {
        assert_eq!(Ok(()), check_buffer_overwrite(0x18, 0x4, 0x40));
    }

    #[test]
    fn check_buffer_overwrite_oob_offset() {
        assert!(check_buffer_overwrite(0x100, 0x1, 0x60).is_err());
    }

    #[test]
    fn check_buffer_overwrite_oob_combined() {
        assert!(check_buffer_overwrite(0x20, 0x10, 0x28).is_err());
    }
}
