#!/bin/bash

module() { 
	export TERMWIDTH=`/bin/stty size 2>&1 | cut -d" " -f2`;
	eval `$RSMODULES_INSTALL_DIR/rsmodules bash,$TERMWIDTH $*`;
}

export MODULEPATH=""
export RSMODULES_INSTALL_DIR=""
export LOADEDMODULES=""
export PYTHONPATH="$RSMODULES_INSTALL_DIR:$PYTHONPATH"
export PERL5LIB="$RSMODULES_INSTALL_DIR:$PERL5LIB"

#
# Bash commandline completion (bash 3.0 and above) for Modules 3.2.10
#
_module_avail() {
    $RSMODULES_INSTALL_DIR/rsmodules noshell avail
}

_module_not_yet_loaded() {
    comm -23  <(_module_avail|sort)  <(tr : '\n' <<<${LOADEDMODULES}|sort)
}

_module_long_arg_list() {
    local cur="$1" i

    if [[ ${COMP_WORDS[COMP_CWORD-2]} == sw* ]]
    then
        COMPREPLY=( $(compgen -W "$(_module_not_yet_loaded)" -- "$cur") )
        return
    fi
    for ((i = COMP_CWORD - 1; i > 0; i--))
    do case ${COMP_WORDS[$i]} in
       add|load)
        COMPREPLY=( $(compgen -W "$(_module_not_yet_loaded)" -- "$cur") )
        break;;
       rm|remove|unload|switch|swap)
        COMPREPLY=( $(IFS=: compgen -W "${LOADEDMODULES}" -- "$cur") )
        break;;
       esac
    done
}

_module() {
    local cur="$2" prev="$3" cmds opts

    COMPREPLY=()

    cmds="available \
          list load purge info \
          unload"

    opts="-h --help"

    case "$prev" in
    info|load)    COMPREPLY=( $(compgen -W "$(_module_not_yet_loaded)" -- "$cur") );;
    unload)
            COMPREPLY=( $(IFS=: compgen -W "${LOADEDMODULES}" -- "$cur") );;
    *) if test $COMP_CWORD -gt 2
       then
        _module_long_arg_list "$cur"
       else
        case "$cur" in
        # The mappings below are optional abbreviations for convenience
        ls)    COMPREPLY="list";;    # map ls -> list
        r*)    COMPREPLY="rm";;    # also covers 'remove'
        sw*)    COMPREPLY="switch";;

        -*)    COMPREPLY=( $(compgen -W "$opts" -- "$cur") );;
        *)    COMPREPLY=( $(compgen -W "$cmds" -- "$cur") );;
        esac
       fi;;
    esac
}
complete -o default -F _module module
