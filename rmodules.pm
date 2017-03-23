sub module {
    eval `$ENV{"RMODULES_INSTALL_DIR"}/rmodules perl @_`;

}

if (! defined $ENV{"MODULEPATH"} ) {
    $ENV{"MODULEPATH"} = "";
}

if (! defined $ENV{"LOADEDMODULES"} ) {
    $ENV{"LOADEDMODULES"} = "";
}

1;
