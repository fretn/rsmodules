#!/bin/csh

alias module 'setenv TERMWIDTH `stty size | cut -d" " -f2` ; eval `$RMODULES_INSTALL_DIR/target/debug/rmodules csh,$TERMWIDTH '*'` ;'
setenv MODULEPATH "/home/frlae/rust/rmodules/modulespath:/home/frlae/rust/rmodules/modulespath2"
setenv RMODULES_INSTALL_DIR "/home/frlae/rust/rmodules/"
setenv LOADEDMODULES ""
