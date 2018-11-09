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
        make && make install
    )
)
sudo cp -a build/libhdfs3/libhdfs3/build/* libhdfs3/.
