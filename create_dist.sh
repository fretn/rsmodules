#!/bin/bash

./build_static.sh

VERSION=`cat Cargo.toml  | grep version | awk -F' = ' '{ print $2 }' | sed -e 's/^"//' -e 's/"$//'`
DIR="rmodules_$VERSION"

mkdir $DIR 
cp rmodules setup_rmodules.sh setup_rmodules.csh rmodules.py rmodules.pm $DIR
cp -R examples/ $DIR
cp -R tools/ $DIR
tar -zcvf "$DIR.tar.gz" $DIR
rm -rf "$DIR"
if [ ! -d "releases" ]; then
	mkdir releases
fi
mv "$DIR.tar.gz" releases/
