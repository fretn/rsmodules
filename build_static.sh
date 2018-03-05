#!/bin/bash

if [ "$(uname)" == "Darwin" ]; then
	cargo build --release
	strip target/release/rsmodules
	cp target/release/rsmodules .
elif [ "$(expr substr $(uname -s) 1 5)" == "Linux" ]; then
	TOOLCHAIN="x86_64-unknown-linux-musl"
	# first add the target:
	if [ "`rustup show | \grep $TOOLCHAIN`" != "$TOOLCHAIN" ]; then
		rustup target add x86_64-unknown-linux-musl
	fi
	# then build
	cargo build --release --target=$TOOLCHAIN
	strip target/$TOOLCHAIN/release/rsmodules
	cp target/$TOOLCHAIN/release/rsmodules .
fi


