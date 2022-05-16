#!/bin/bash

rm rsmodules
cargo build
cp target/debug/rsmodules rsmodules

