#!/bin/sh

export RUSTFLAGS="-Ctarget-cpu=sandybridge -Ctarget-feature=+aes,+sse2,+sse4.1,+ssse3"
export RUST_BACKTRACE=full
export PKG_CONFIG_ALLOW_CROSS=1
cargo build --release --target=x86_64-pc-windows-gnu --example windows
