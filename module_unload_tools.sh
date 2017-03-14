#!/usr/bin/env bash

# while unloading, we don't auto unload others
module() { 
    echo ""
}


# while unloading, append-path and prepend-path are actually remove-path
# also the rust code needs to replace "export BLAH="lmkj" by unset BLAH before
# running the unload module script
append_path ()  { 
    a="$1"
    eval a=\$$a

    export "$1"=`echo -n $a | awk -v RS=: -v ORS=: '$0 != "'$2'"' | sed 's/:$//'`
}

prepend_path () { 
    a="$1"
    eval a=\$$a

    export "$1"=`echo -n $a | awk -v RS=: -v ORS=: '$0 != "'$2'"' | sed 's/:$//'`
}

# when we unload, we do not execute remove-path
remove_path ()  { 
    echo ""
}
