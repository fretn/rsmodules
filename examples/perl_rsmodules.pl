#!/usr/bin/perl

use rsmodules;

module("load","blast");

print($ENV{"LOADEDMODULES"} . "\n");

module("list","");

# this var is set with setenv("SOMEVAR","value") in the modulescript
print($ENV{"SOMEVAR"});
