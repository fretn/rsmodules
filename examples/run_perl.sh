#!/bin/bash

export PERL5LIB="${PERL5LIB}:$RMODULES_INSTALL_DIR"

perl perl_rmodules.pl
