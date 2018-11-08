#!/bin/sh

mkdir -p build
cd build
git clone https://github.com/ContinuumIO/libhdfs3-downstream.git libhdfs3
cd libhdfs3/libhdfs3
cmake
sudo cp -a dist/include /usr/include
sudo cp -a dist/lib /usr/lib/