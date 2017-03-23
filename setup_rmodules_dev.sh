#!/bin/bash

source /home/frlae/rust/rmodules/setup_rmodules.sh

module() { 
	export TERMWIDTH=`/bin/stty size | cut -d" " -f2`;
	eval `$RMODULES_INSTALL_DIR/target/debug/rmodules bash,$TERMWIDTH $*`;
}

export MODULEPATH="/usr/local/modules:/home/frlae/rust/rmodules/modulespath:/home/frlae/rust/rmodules/modulespath2"
export RMODULES_INSTALL_DIR="/home/frlae/rust/rmodules/"
export LOADEDMODULES=""
