#!/bin/bash

module() { 
    eval `$RMODULES_INSTALL_DIR/target/debug/rmodules bash $*`; 
}

export MODULEPATH="/home/frlae/rust/rmodules/modulespath:/home/frlae/rust/rmodules/modulespath2"
export RMODULES_INSTALL_DIR="/home/frlae/rust/rmodules/"
export LOADEDMODULES=""
