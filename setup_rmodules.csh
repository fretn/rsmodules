#!/bin/csh

alias module 'eval `/home/frlae/rust/rmodules/target/debug/rmodules csh '*'` ;'
setenv MODULEPATH "/home/frlae/rust/rmodules/modulespath:/home/frlae/rust/rmodules/modulespath2"
setenv RMODULES_INSTALL_DIR "/home/frlae/rust/rmodules/"
