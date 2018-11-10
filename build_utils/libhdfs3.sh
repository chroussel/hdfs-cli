#!/bin/sh
set +ev

num_proc=1
# If PARALLEL_BUILD environment variable is set then parallel build is enabled
if [ "${PARALLEL_BUILD:-}" != "" ]; then
    # If PARALLEL_BUILD content is a number then use it as number of parallel jobs
    if [ ! -z "${PARALLEL_BUILD##*[!0-9]*}" ]; then
        num_proc=${PARALLEL_BUILD}
    else
        # Try to determine the number of available CPUs
        if command_exists nproc; then
            num_proc=$(nproc)
            elif command_exists sysctl; then
            num_proc=$(sysctl -n hw.ncpu)
        fi
    fi
fi

mkdir -p build
(
    cd build
    git clone https://github.com/ContinuumIO/libhdfs3-downstream.git libhdfs3
    cd libhdfs3/libhdfs3
    mkdir -p build
    (
        cd build
        cmake ..
        make -j $(num_proc)
        sudo make install
    )
)