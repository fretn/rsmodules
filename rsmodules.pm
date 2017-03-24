sub module {
    eval `$ENV{"RSMODULES_INSTALL_DIR"}/rsmodules perl @_`;

}

if (! defined $ENV{"MODULEPATH"} ) {
    $ENV{"MODULEPATH"} = "";
}

if (! defined $ENV{"LOADEDMODULES"} ) {
    $ENV{"LOADEDMODULES"} = "";
}

1;
