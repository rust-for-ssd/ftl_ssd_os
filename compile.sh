#!/bin/sh
cargo b --profile small
ar x ./target/target/small/libftl_ssd_os.a
find . -type f -name "ftl_ssd_os*.o" -exec cp {} ../ssd_os/app/programs/build_rs/rs_ftl.o \;
cp ftl_ssd_os.conn ../ssd_os/app/programs/rs_ftl.conn
cp ftl_ssd_os.pipe ../ssd_os/app/programs/rs_ftl.pipe