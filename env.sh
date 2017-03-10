#!/bin/bash

module() { 
    eval `/home/frlae/rust/rmodules/target/debug/rmodules bash $*`; 
}
