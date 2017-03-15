#!/bin/csh

alias module 'eval `$RMODULES_INSTALL_DIR/target/x86_64-unknown-linux-musl/debug/rmodules csh ' \!'*` '

setenv MODULEPATH "/home/frlae/rust/rmodules/modulespath:/home/frlae/rust/rmodules/modulespath2"
setenv RMODULES_INSTALL_DIR "/home/frlae/rust/rmodules/"
setenv LOADEDMODULES ""
