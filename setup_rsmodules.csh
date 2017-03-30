#!/bin/csh

alias module 'setenv TERMWIDTH `stty size |& tee /dev/null | cut -d" " -f2` ; eval `$RSMODULES_INSTALL_DIR/rsmodules csh,$TERMWIDTH ' \!'*` '
setenv MODULEPATH ""
setenv RSMODULES_INSTALL_DIR ""
#setenv LOADEDMODULES ""
if (! $?PYTHONPATH ) then
        setenv PYTHONPATH "$RSMODULES_INSTALL_DIR"
else
        setenv PYTHONPATH "$RSMODULES_INSTALL_DIR:$PYTHONPATH"
endif
if (! $?PERL5LIB ) then
        setenv PERL5LIB "$RSMODULES_INSTALL_DIR"
else
        setenv PERL5LIB "$RSMODULES_INSTALL_DIR:$PERL5LIB"
endif

# this should be a function, so everytime it is called the info is updated
set mod_av="`$RSMODULES_INSTALL_DIR/rsmodules noshell avail`"

complete module \
'n#load#$mod_av#' \
'n#info#$mod_av#' \
'n#unload#$mod_av#' \
'p#1#(info load unload available \
    purge list)#'