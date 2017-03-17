#!/bin/bash

module() { 
	export TERMWIDTH=`/bin/stty size | cut -d" " -f2`;
	eval `$RMODULES_INSTALL_DIR/target/debug/rmodules bash,$TERMWIDTH $*`;
}

export MODULEPATH="/home/frlae/rust/rmodules/modulespath:/home/frlae/rust/rmodules/modulespath2"
export RMODULES_INSTALL_DIR="/home/frlae/rust/rmodules/"
export LOADEDMODULES=""
