#!/bin/sh +e

mkdir -p build
cd build
wget https://github.com/google/googletest/archive/release-1.8.1.tar.gz
tar xf release-1.8.1.tar.gz
cd googletest-release-1.8.1
cmake .
make
sudo make install