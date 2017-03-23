#!/bin/csh

alias module 'setenv TERMWIDTH `stty size |& tee /dev/null | cut -d" " -f2` ; eval `$RMODULES_INSTALL_DIR/rmodules csh,$TERMWIDTH ' \!'*` '
setenv MODULEPATH ""
setenv RMODULES_INSTALL_DIR ""
setenv LOADEDMODULES ""
setenv PYTHONPATH "$RMODULES_INSTALL_DIR:$PYTHONPATH"
setenv PERL5LIB "$RMODULES_INSTALL_DIR:$PERL5LIB"

# this should be a function, so everytime it is called the info is updated
set mod_av="`$RMODULES_INSTALL_DIR/rmodules noshell avail`"

complete module \
'n#load#$mod_av#' \
'n#info#$mod_av#' \
'n#unload#$mod_av#' \
'p#1#(info load unload available \
    purge list)#'
