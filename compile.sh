#!/bin/sh
cargo b --profile small
ar x ./target/target/small/libftl_ssd_os.a
find . -type f -name "ftl_ssd_os*.o" -exec cp {} ../ssd_os/app/programs/build_rs/ftl_ssd_os.o \;
cp ftl_ssd_os.conn ../ssd_os/app/programs/ftl_ssd_os.conn
cp ftl_ssd_os.pipe ../ssd_os/app/programs/ftl_ssd_os.pipe