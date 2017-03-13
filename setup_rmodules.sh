#!/bin/bash

module() { 
    eval `/home/frlae/rust/rmodules/target/debug/rmodules bash $*`; 
}

export MODULEPATH="/home/frlae/rust/rmodules/modulespath:/home/frlae/rust/rmodules/modulespath2"
export RMODULES_INSTALL_DIR="/home/frlae/rust/rmodules/"
