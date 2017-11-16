#!/bin/bash

./build_static.sh

VERSION=`cat Cargo.toml  | grep version | awk -F' = ' '{ print $2 }' | sed -e 's/^"//' -e 's/"$//'`
DIR="rsmodules_$VERSION"

mkdir $DIR 
cp rsmodules setup_rsmodules.sh setup_rsmodules.csh rsmodules.py rsmodules.pm $DIR
cp -R examples/ $DIR
cp -R tools/ $DIR
cp README.md $DIR
tar --owner=root --group=root -zcvf "$DIR.tar.gz" $DIR
rm -rf "$DIR"
if [ ! -d "releases" ]; then
	mkdir releases
fi
mv "$DIR.tar.gz" releases/
