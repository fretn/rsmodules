#!/bin/bash

source /home/frlae/rust/rmodules/setup_rsmodules.sh

if [ "`hostname`" == "modimo" ]; then
	module() {
		export TERMWIDTH=`/bin/stty size | cut -d" " -f2`;
		eval `$RSMODULES_INSTALL_DIR/target/debug/rsmodules bash,$TERMWIDTH $*`;
	}

	export MODULEPATH="/usr/local/modules:/home/frlae/rust/rmodules/modulespath:/home/frlae/rust/rmodules/modulespath2"
	export RSMODULES_INSTALL_DIR="/home/frlae/rust/rmodules/"
	export LOADEDMODULES=""
fi
if [ "`hostname`" == "midas.psb.ugent.be" ]; then
	source /software/shared/apps/x86_64/rsmodules/0.2.0/setup_rsmodules.sh
fi
