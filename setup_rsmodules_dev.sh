#!/bin/bash

module() {
	export TERMWIDTH=`/bin/stty size 2>&1 | cut -d" " -f2`;
	eval `$RSMODULES_INSTALL_DIR/rsmodules bash,$TERMWIDTH $*`;
}

export MODULEPATH=""
export RSMODULES_INSTALL_DIR=""

if [ "`hostname`" == "pad39a.psb.ugent.be" ]; then
	#export MODULEPATH="/usr/local/modules:/Users/frlae/rust/rmodules/modulespath:/Users/frlae/rust/rmodules/modulespath2"
	export MODULEPATH="/Users/frlae/rust/rmodules/modulespath:/Users/frlae/rust/rmodules/modulespath2"
	export RSMODULES_INSTALL_DIR="/Users/frlae/rust/rmodules/"

	module() {
		export TERMWIDTH=`/bin/stty size | cut -d" " -f2`;
		eval `$RSMODULES_INSTALL_DIR/target/debug/rsmodules bash,$TERMWIDTH $*`;
	}

	export -f module

	update_modules_cache() {
		export TERMWIDTH=`/bin/stty size 2>&1 | cut -d" " -f2`;
		$RSMODULES_INSTALL_DIR/target/debug/rsmodules progressbar,$TERMWIDTH cache make;
	}

	export -f update_modules_cache

	#export LOADEDMODULES=""
fi

#export LOADEDMODULES=""
if [ -z ${PYTHONPATH+x} ]; then
	export PYTHONPATH="$RSMODULES_INSTALL_DIR"
else
	export PYTHONPATH="$RSMODULES_INSTALL_DIR:$PYTHONPATH"
fi
if [ -z ${PERL5LIB+x} ]; then
	export PERL5LIB="$RSMODULES_INSTALL_DIR"
else
	export PERL5LIB="$RSMODULES_INSTALL_DIR:$PERL5LIB"
fi

if [ -f ~/.rsmodules_autoload ]; then
	source ~/.rsmodules_autoload
fi

# cleanup old tmp files from crashed rsmodules sessions
find ~/.rsmodulestmp* -mtime +1 -delete > /dev/null 2>&1
# also cleanup empty tmp files
find ~/.rsmodulestmp* -empty -delete > /dev/null 2>&1

if [ ${BASH_VERSINFO:-0} -ge 3 ]; then
	#
	# Bash commandline completion (bash 3.0 and above)
	#
	_module_avail() {
		$RSMODULES_INSTALL_DIR/rsmodules noshell avail
	}

	_module_not_yet_loaded() {
		comm -23  <(_module_avail|sort)  <(tr : '\n' <<<${LOADEDMODULES}|sort)
		# workaround for galaxy bug
		#_module_avail
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
			rm|unload|switch)
				COMPREPLY=( $(IFS=: compgen -W "${LOADEDMODULES}" -- "$cur") )
				break;;
			autoload)
				COMPREPLY=( $(compgen -W "$(_module_avail)" -- "$cur") )
				break;;
			esac
		done
	}

	_module() {
		local cur="$2" prev="$3" cmds opts

		COMPREPLY=()

		cmds="available \
			list load purge info cd edit \
			unload readme cache add rm \
			autoload refurbish undo refresh delete create"

		opts="-h --help"

		case "$prev" in
		load)    COMPREPLY=( $(compgen -W "$(_module_not_yet_loaded)" -- "$cur") );;
		unload)
				COMPREPLY=( $(IFS=: compgen -W "${LOADEDMODULES}" -- "$cur") );;
		delete|readme|info|cd|edit)
				      COMPREPLY=( $(compgen -W "$(_module_avail)" -- "$cur") );;
		cache)	COMPREPLY=( $(IFS=: compgen -W "make:add:edit:delete" -- "$cur") );;
		autoload)
				COMPREPLY=( $(IFS=: compgen -W "append:prepend:list:purge:remove" -- "$cur") );;
		*)  if test $COMP_CWORD -gt 2
			then
				_module_long_arg_list "$cur"
			else
				case "$cur" in
				# The mappings below are optional abbreviations for convenience
				ls)    COMPREPLY="list";;    # map ls -> list
				#r*)    COMPREPLY="rm";;    # also covers 'remove'
				sw*)    COMPREPLY="switch";;

				-*)    COMPREPLY=( $(compgen -W "$opts" -- "$cur") );;
				*)    COMPREPLY=( $(compgen -W "$cmds" -- "$cur") );;
				esac
			fi;;
		esac
	}
	complete -o default -F _module module
	export -f module
fi

