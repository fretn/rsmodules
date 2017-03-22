#!/bin/csh

alias module 'setenv TERMWIDTH `stty size |& tee /dev/null | cut -d" " -f2` ; eval `$RMODULES_INSTALL_DIR/rmodules csh,$TERMWIDTH ' \!'*` '
setenv MODULEPATH "/software/shared/rmodulefiles/"
setenv RMODULES_INSTALL_DIR "/software/shared/apps/x86_64/rmodules/0.2.0/"
setenv LOADEDMODULES ""

# this should be a function, so everytime it is called the info is updated
set mod_av="`$RMODULES_INSTALL_DIR/rmodules noshell avail`"

complete module \
'n#load#$mod_av#' \
'n#info#$mod_av#' \
'n#unload#$mod_av#' \
'p#1#(info load unload available \
    purge list)#'
