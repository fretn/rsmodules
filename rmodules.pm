sub module {
    eval `/home/frlae/rust/rmodules/rmodules perl @_`;

}

if (! defined $ENV{"MODULEPATH"} ) {
    $ENV{"MODULEPATH"} = "";
}

if (! defined $ENV{"LOADEDMODULES"} ) {
    $ENV{"LOADEDMODULES"} = "";
}

1;
