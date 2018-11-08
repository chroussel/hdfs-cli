#!/bin/sh

# lib requirements
sudo apt-get -qq update
sudo apt-get install -y libgtest-dev libboost-dev libxml2-dev cmake

mkdir -p deps
cd deps
(
    sudo wget https://github.com/google/googletest/archive/release-1.8.1.tar.gz
    sudo tar xf release-1.8.1.tar.gz
    cd googletest-release-1.8.1
    sudo cmake -DBUILD_SHARED_LIBS=ON .
    sudo make
    sudo cp -a include/gtest /usr/include
    sudo cp -a libgtest_main.so libgtest.so /usr/lib/
)

(
    git clone https://github.com/ContinuumIO/libhdfs3-downstream.git libhdfs3
    cd libhdfs3/libhdfs3
    cmake
)