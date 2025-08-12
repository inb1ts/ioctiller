# Ioctiller

Simple tool to assist in manually testing Windows drivers. This is essentially just a wrapper around `DeviceIoControl`, where the input buffer size and contents are specified in a TOML file. The aim of this is to simply ease testing for vulns without having to recompile frequently or repetitively move tools between host and target.

The workflow I currently use this in is to have the config TOML in a folder that is shared between the host and target machine. I can then update the TOML whilst working through the driver in Binja, and then switch over to actually send requests quite quickly.

## Usage

```bash
ioctiller.exe <CONFIG PATH>
```

Once the tool has been run, it will prompt the user to pick from the IOCTLs available in the config file.

## Config format

### `ioctls`
The config file should be build up of a table of `ioctls` records. This specifies:
- `name`: The name you want to use to identify the IOCTL when running the tool.
- `code`: The actual I/O Control Code that willl be used in the `DeviceIoControl` call.
- `input_buffer_size`: The input buffer size that will be passed to `DeviceIoControl`.
- `output_buffer_size`: The output buffer size that will be passed to `DeviceIoControl`.

### `input_buffer_content`

Each `ioctls` section will then be optionally followed by as many `input_buffer_content` entries as required to fill out the parts of the input buffer that are desired. If no `input_buffer_content` entries are specified, the input buffer will just be zeroed.

These entries specify:
- `offset`: The offset, in bytes, from the beginning of the input buffer, that this entry should be written
- `type`: A string of the type of value that should be written. This can be one of the following:
- - `"U8"`
- - `"U16"`
- - `"U32"`
- - `"U64"`
- - `"String8"`
- - `"FILL"`
- `value`: The actual value of the entry to be written to the input buffer.
- `length` (for `fill` only): How many bytes should be filled with the `char` in `value`.

If there is a mismatch between the type specified and the value provided, the tool will error (this uses [serde](https://serde.rs/) for this).

### Example

```toml
[[ioctls]]
name = "IOCTL_1"
code = 0x10000
input_buffer_size = 64
output_buffer_size = 128
input_buffer_content = [
    { offset = 0x10, type="U8", value=0x41 }
]

[[ioctls]]
name = "IOCTL_2"
code = 0x220008
input_buffer_size = 32
output_buffer_size = 64
input_buffer_content = [
    { offset=0x0, type="FILL", value=0x41 },
    { offset=0x38, type="STRING8", value="foobar" }
]

[[ioctls]]
name = "IOCTL_3"
code = 0x22000C
input_buffer_size = 0
output_buffer_size = 256
```

