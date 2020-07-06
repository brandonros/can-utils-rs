# can-utils-rs
isotprecv + isotpsend in Rust with userspace USB device drivers

## Usage (Mac/Linux)

1. Download latest release archive from https://github.com/brandonros/can-utils-rs/releases
1. Extract archive.
1. Open 3 terminals.

In terminal 1:

1. `./server`

In terminal 2:

1. `./isotprecv-loop.sh`

In terminal 3:

1. `./isotpsend.sh "10 03"`

## Usage (Windows)

1. Download latest relase archive from https://github.com/brandonros/can-utils-rs/releases
1. Extract archive.
1. Open 3 PowerShell instances.

In PowerShell instance 1:

1. `cd extracted_archive_directory`
1. `(new-object System.Net.WebClient).DownloadFile("https://frippery.org/files/busybox/busybox.exe","$(pwd)/busybox.exe")`
1. `./server.exe`

In PowerShell instance 2:

1. `cd extracted_archive_directory`
1. `./busybox.exe sh isotprecv-loop.sh`

In PowerShell instance 3:

1. `cd extracted_archive_directory`
1. `./busybox.exe sh isotpsend.sh "10 03"`
