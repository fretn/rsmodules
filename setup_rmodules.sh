#!/bin/bash

module() { 
	export TERMWIDTH=`/bin/stty size | cut -d" " -f2`;
	eval `$RMODULES_INSTALL_DIR/rmodules bash,$TERMWIDTH $*`;
}

export MODULEPATH="/software/shared/rmodulefiles/"
export RMODULES_INSTALL_DIR="/software/shared/apps/x86_64/rmodules/0.2.0/"
export LOADEDMODULES=""

#
# Bash commandline completion (bash 3.0 and above) for Modules 3.2.10
#
_module_avail() {
    export RMODULES_AV_LIST=1; /software/shared/apps/x86_64/rmodules/0.2.0/rmodules bash avail 2>&1 | sed '
        /:$/d;
        /:ERROR:/d;
        s#^\(.*\)/\(.\+\)(default)#\1\n\1\/\2#;
        s#/(default)##g;
        s#/*$##g;'
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
    add|load)    COMPREPLY=( $(compgen -W "$(_module_not_yet_loaded)" -- "$cur") );;
    rm|remove|unload|switch|swap)
            COMPREPLY=( $(IFS=: compgen -W "${LOADEDMODULES}" -- "$cur") );;
    unuse)        COMPREPLY=( $(IFS=: compgen -W "${MODULEPATH}" -- "$cur") );;
    use|*-a*)    ;;            # let readline handle the completion
    -u|--userlvl)    COMPREPLY=( $(compgen -W "novice expert advanced" -- "$cur") );;
    display|help|show|whatis)
            COMPREPLY=( $(compgen -W "$(_module_avail)" -- "$cur") );;
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
