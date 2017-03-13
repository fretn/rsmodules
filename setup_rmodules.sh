#!/bin/bash

module() { 
    eval `/home/frlae/rust/rmodules/target/debug/rmodules bash $*`; 
}

export MODULEPATH="./modulespath:./modulespath2"
