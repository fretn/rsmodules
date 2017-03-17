#!/bin/bash

# first add the target:
# rustup target add x86_64-unknown-linux-musl

# then build
cargo build --release --target=x86_64-unknown-linux-musl
strip target/x86_64-unknown-linux-musl/release/rmodules
cp target/x86_64-unknown-linux-musl/release/rmodules .
