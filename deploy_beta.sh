#!/bin/bash

./build_linux_static.sh
scp rsmodules root@$RSMODULES_DEPLOY_HOST:/software/shared/apps/x86_64/rsmodules/beta/rsmodules


