#!/bin/bash

TOOLCHAIN="x86_64-unknown-linux-musl"
# first add the target:
if [ "`rustup show | \grep $TOOLCHAIN`" != "$TOOLCHAIN" ]; then
	rustup target add x86_64-unknown-linux-musl
fi

# then build
cargo build --release --target=$TOOLCHAIN
strip target/$TOOLCHAIN/release/rmodules
cp target/$TOOLCHAIN/release/rmodules .
