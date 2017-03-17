#!/bin/bash

module() { 
	export TERMWIDTH=`/bin/stty size | cut -d" " -f2`;
	eval `$RMODULES_INSTALL_DIR/rmodules bash,$TERMWIDTH $*`;
}

export MODULEPATH="/software/shared/rmodulefiles/"
export RMODULES_INSTALL_DIR="/software/shared/apps/x86_64/rmodules/0.2.0/"
export LOADEDMODULES=""
