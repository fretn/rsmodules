#!/bin/bash

TOOLCHAIN="x86_64-unknown-linux-musl"
# first add the target:
if [ "`rustup show | \grep $TOOLCHAIN`" != "$TOOLCHAIN" ]; then
	rustup target add x86_64-unknown-linux-musl
fi

STRIP="strip"

if [ "$(uname)" == "Darwin" ]; then
	STRIP="x86_64-linux-musl-strip"

	MUSL_INSTALLED=`brew list -l | \grep -c musl-cross`

	if [ "$MUSL_INSTALLED" != "1" ]; then
		brew install FiloSottile/musl-cross/musl-cross

		if [ ! -f /usr/local/bin/musl-gcc ]; then
			sudo ln -s /usr/local/bin/x86_64-linux-musl-cc /usr/local/bin/musl-gcc
		fi
	fi

	if [ ! -d .cargo ]; then
		mkdir .cargo
	fi

	if [ ! -f .cargo/config ]; then
		echo "[target.x86_64-unknown-linux-musl]" > .cargo/config
		echo "linker = \"x86_64-linux-musl-gcc\"" >> .cargo/config
	fi


fi

cargo build --release --target=$TOOLCHAIN
$STRIP target/$TOOLCHAIN/release/rsmodules
cp target/$TOOLCHAIN/release/rsmodules .


