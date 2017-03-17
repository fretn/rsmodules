#!/bin/csh

alias module 'setenv TERMWIDTH `stty size | cut -d" " -f2` ; eval `$RMODULES_INSTALL_DIR/rmodules csh,$TERMWIDTH ' \!'*` '
setenv MODULEPATH "/software/shared/rmodulefiles/"
setenv RMODULES_INSTALL_DIR "/software/shared/apps/x86_64/rmodules/0.2.0/"
setenv LOADEDMODULES ""
