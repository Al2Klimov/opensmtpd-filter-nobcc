## About

OpenSMTPd filter which rejects eMails sent via BCC (mostly spam).

## Build

Compile like any other Rust program: `cargo build -r`

Find the resulting binary directly under `target/release/`.

## Usage

Integrate this filter into smtpd.conf(5).
