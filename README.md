# can-utils-rs
isotprecv + isotpsend in Rust with userspace USB device drivers

## Usage (Mac/Linux)

1. `./target/release/server`

In another terminal:

1. `./scripts/isotprecv-loop.sh`

In another terminal:

1. `./scripts/isotpsend.sh "10 03"`

## Usage (Windows)

1. Download latest artifact archive from https://github.com/brandonros/can-utils-rs/actions
1. Extract archive.
1. Open 3 PowerShell instances.

In PowerShell instance 1:

1. `cd extracted_archive_directory`
1. `(new-object System.Net.WebClient).DownloadFile('https://frippery.org/files/busybox/busybox.exe','busybox.exe')`
1. `busybox.exe sh`
1. `./server.exe`

In PowerShell instance 2:

1. `cd extracted_archive_directory`
1. `busybox.exe sh`
1. `./scripts/isotprecv-loop.sh`

In PowerShell instance 3:

1. `cd extracted_archive_directory`
1. `busybox.exe sh`
1. `./scripts/isotpsend.sh "10 03"`
