#!/bin/sh

# Set default path if not provided
SSD_OS_PATH=${SSD_OS_PATH:-../ssd_os}
APP_NAME=${APP_NAME:-"connector_per_component"}


rm *.o
# cargo clean
cargo b --profile small
ar x ./target/target/small/libftl_ssd_os.a

if riscv32-none-elf-nm ftl_ssd_os*.o | grep -q "this_doesnt_exist"; then
    echo "❌ Error: Forbidden symbol 'this_doesnt_exist' found in binary."
    exit 1
fi

find . -type f -name "ftl_ssd_os*.o" -exec cp {} "$SSD_OS_PATH/app/programs/build_rs/rs_ftl.o" \;
cp "src/apps/$APP_NAME/$APP_NAME.conn" "$SSD_OS_PATH/app/programs/rs_ftl.conn"
cp "src/apps/$APP_NAME/$APP_NAME.pipe" "$SSD_OS_PATH/app/programs/rs_ftl.pipe"
