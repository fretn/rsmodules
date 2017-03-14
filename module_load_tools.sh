#!/usr/bin/env bash

module() { 
    eval `/home/frlae/rust/rmodules/target/debug/rmodules bash $*`;
}

append_path ()  { 
    a="$1"
    eval a=\$$a

    export "$1"="$a:$2"
}

prepend_path () { 
    a="$1"
    eval a=\$$a

    export "$1"="$2:$a"
}

remove_path ()  { 
    a="$1"
    eval a=\$$a

    export "$1"=`echo -n $a | awk -v RS=: -v ORS=: '$0 != "'$2'"' | sed 's/:$//'`
}

prepend_path LOADEDMODULES "$1"

#export () {
#    unset TEST
#}
