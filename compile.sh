#!/bin/sh

# Set default path if not provided
SSD_OS_PATH=${SSD_OS_PATH:-../ssd_os}
APP_NAME=${APP_NAME:-"connector_per_component"}



rm *.o
# cargo clean
cargo b --profile small --features=$APP_NAME
ar x ./target/target/small/libftl_ssd_os.a

if riscv32-none-elf-nm ftl_ssd_os*.o | grep -q "this_doesnt_exist"; then
    echo "❌ Error: Forbidden symbol 'this_doesnt_exist' found in binary."
    exit 1
fi

find . -type f -name "ftl_ssd_os*.o" -exec cp {} "$SSD_OS_PATH/app/programs/build_rs/rs_ftl.o" \;

./conn_checker.sh "src/apps/$APP_NAME/connectors.conn" || exit 1
./pipe_checker.sh "src/apps/$APP_NAME/pipelines.pipe" || exit 1
./pipe_name_checker.sh "src/apps/$APP_NAME/connectors.conn" "src/apps/$APP_NAME/pipelines.pipe" || exit 1

cp "src/apps/$APP_NAME/connectors.conn" "$SSD_OS_PATH/app/programs/rs_ftl.conn"
cp "src/apps/$APP_NAME/pipelines.pipe" "$SSD_OS_PATH/app/programs/rs_ftl.pipe"

echo "Built app: $APP_NAME"
