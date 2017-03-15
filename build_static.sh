#!/bin/bash

# first add the target:
# rustup target add x86_64-unknown-linux-musl

# then build
cargo build --target=x86_64-unknown-linux-musl
