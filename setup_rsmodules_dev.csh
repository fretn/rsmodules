#!/bin/csh

#source ~/rust/rmodules/setup_rsmodules.csh

if ("`hostname`" == "modimo.psb.ugent.be") then
	alias module 'setenv TERMWIDTH `stty size |& tee /dev/null | cut -d" " -f2` ; eval `$RSMODULES_INSTALL_DIR/target/debug/rsmodules csh,$TERMWIDTH ' \!'*` '
	setenv MODULEPATH "/usr/local/modules:~/rust/rmodules/modulespath:~/rust/rmodules/modulespath2"
	setenv RSMODULES_INSTALL_DIR "~/rust/rmodules/"
	#setenv LOADEDMODULES ""
endif
if ("`hostname`" == "midas.psb.ugent.be") then
	source /software/shared/apps/x86_64/rsmodules/0.2.0/setup_rsmodules.csh
endif

alias module 'setenv TERMWIDTH `stty size |& tee /dev/null | cut -d" " -f2` ; eval `$RSMODULES_INSTALL_DIR/rsmodules csh,$TERMWIDTH ' \!'*` '
alias update_modules_cache 'setenv TERMWIDTH `stty size |& tee /dev/null | cut -d" " -f2` ; $RSMODULES_INSTALL_DIR/rsmodules progressbar,$TERMWIDTH cache make '

