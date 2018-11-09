#!/bin/sh
set +ev

mkdir -p build
(
    cd build
    git clone https://github.com/ContinuumIO/libhdfs3-downstream.git libhdfs3
    cd libhdfs3/libhdfs3
    mkdir -p build
    (
        cd build
        cmake ..
        make -j 2
        sudo make install
    )
)