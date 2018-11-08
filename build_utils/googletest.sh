#!/bin/sh

mkdir -p build
cd build
sudo wget https://github.com/google/googletest/archive/release-1.8.1.tar.gz
sudo tar xf release-1.8.1.tar.gz
cd googletest-release-1.8.1
sudo cmake
sudo make
sudo cp -a include/gtest /usr/include
sudo cp -a libgtest_main.so libgtest.so /usr/lib/