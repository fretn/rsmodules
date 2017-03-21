#!/bin/csh

alias module 'setenv TERMWIDTH `stty size | cut -d" " -f2` ; eval `$RMODULES_INSTALL_DIR/rmodules csh,$TERMWIDTH ' \!'*` '
setenv MODULEPATH "/software/shared/rmodulefiles/"
setenv RMODULES_INSTALL_DIR "/software/shared/apps/x86_64/rmodules/0.2.0/"
setenv LOADEDMODULES ""

# this should be a function, so everytime it is called the info is updated
set mod_av=`setenv RMODULES_AV_LIST; $RMODULES_INSTALL_DIR/rmodules csh avail | & sed '/:$/d;/:ERROR:/d;s#^\(.*\)/\(.\+\)(default)#\1\n\1\/\2#;s#/(default)##g;s#/*$##g;'`

complete module \
'n#load#$mod_av#' \
'n#unload#$mod_av#' \
'p#1#(load unload avail \
    purge list)#'
