# can-utils-rs
isotprecv + isotpsend in Rust with userspace USB device drivers

## Usage

1. `cargo build --release`
1. `./target/release/server`
1. `./scripts/isotprecv-loop.sh`
1. `./scripts/isotpsend.sh "10 03"`
