#!/bin/csh

source /home/frlae/rust/rmodules/setup_rsmodules.csh

if ("`hostname`" == "modimo") then
	alias module 'setenv TERMWIDTH `stty size | cut -d" " -f2` ; eval `$RSMODULES_INSTALL_DIR/target/debug/rsmodules csh,$TERMWIDTH '*'` ;'
	setenv MODULEPATH "/usr/local/modules:/home/frlae/rust/rmodules/modulespath:/home/frlae/rust/rmodules/modulespath2"
	setenv RSMODULES_INSTALL_DIR "/home/frlae/rust/rmodules/"
	#setenv LOADEDMODULES ""
endif
if ("`hostname`" == "midas.psb.ugent.be") then
	source /software/shared/apps/x86_64/rsmodules/0.2.0/setup_rsmodules.csh
endif
