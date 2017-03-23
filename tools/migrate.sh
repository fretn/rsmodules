#!/bin/bash

# this is an example to find all tcl module files in a /blah/modulefiles folder
# and migrate them to rhaiscript in /blah/rmodulefiles

for file in `find /software/shared/modulefiles/ -type f`
do
    target="`echo $file | sed 's/modulefiles/rmodulefiles/'`"
    target_filename="`basename $target`"
    target_dirname="`dirname $target`"

    # first : creat target folder
    mkdir -p $target_dirname

    if [ $target_filename != ".version" ] && [ $target_filename != ".modulerc" ]; then
        #echo $target_filename
        `./tcl2rhai.tcl $file > $target`
    fi

    if [ $target_filename == ".modulerc" ]; then
        grep -v "Module" $file | awk '{ print $2 }' > "$target_dirname/.version"
    fi

done
