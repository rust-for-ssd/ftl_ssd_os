#!/bin/sh
cargo b --profile small
ar x ./target/target/small/libssd_os_rust_demo.a
find . -type f -name "ssd_os*.o" -exec cp {} ../ssd_os/app/programs/build_rs/hello_rs.o \;