#!/bin/bash

./build_static.sh

VERSION=`cat Cargo.toml  | grep -m 1 version | awk -F' = ' '{ print $2 }' | sed -e 's/^"//' -e 's/"$//'`
DIR="rsmodules_$VERSION"
OS=`uname | tr '[:upper:]' '[:lower:]'`
MACHINE=`uname -m`

mkdir $DIR 
cp rsmodules setup_rsmodules.sh setup_rsmodules.csh rsmodules.py rsmodules.pm $DIR
cp -R examples/ $DIR
cp -R tools/ $DIR
cp README.md $DIR
tar --owner=root --group=root -zcvf "$DIR""_""$OS""_""$MACHINE.tar.gz" $DIR
rm -rf "$DIR"
if [ ! -d "releases" ]; then
	mkdir releases
fi
mv "$DIR""_""$OS""_""$MACHINE.tar.gz" releases/
