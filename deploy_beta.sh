#!/bin/bash

./build_linux_static_debug.sh

if [ -z ${RSMODULES_DEPLOY_HOST} ]; then
	echo "The variable RSMODULES_DEPLOY_HOST is not found."
	echo "Please enter the name of the host you want to deploy to: "
	read RSMODULES_DEPLOY_HOST
fi
scp rsmodules root@$RSMODULES_DEPLOY_HOST:/software/shared/apps/x86_64/rsmodules/beta/rsmodules


