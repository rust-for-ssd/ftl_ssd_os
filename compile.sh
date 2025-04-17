#!/bin/sh
rm *.o;
# cargo clean;
cargo b --profile small;
ar x ./target/target/small/libftl_ssd_os.a;

if riscv32-none-elf-nm ftl_ssd_os*.o | grep -q "this_doesnt_exist"; then
    echo "❌ Error: Forbidden symbol 'this_doesnt_exist' found in binary."
    exit 1
fi

find . -type f -name "ftl_ssd_os*.o" -exec cp {} ../ssd_os/app/programs/build_rs/rs_ftl.o \;
cp ftl_ssd_os.conn ../ssd_os/app/programs/rs_ftl.conn;
cp ftl_ssd_os.pipe ../ssd_os/app/programs/rs_ftl.pipe;
