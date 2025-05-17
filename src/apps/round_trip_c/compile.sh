#!/bin/sh
SSD_OS_PATH=${SSD_OS_PATH:-../ssd_os}
APP="src/apps/round_trip_c"

# clang \
#   --target=riscv32 \
#   -mcmodel=medany -march=rv32iamfd -mabi=ilp32d \
#   -I"$LIBCLANG_PATH/clang/19/include/"  -Os \
#   -g0 \
#   -c $APP/round_trip_c.c \
#   -o $APP/round_trip_c.o

# clang \
#   --target=riscv32 \
#   -mcmodel=medany -march=rv32iamfd -mabi=ilp32d \
#   -I"$LIBCLANG_PATH/clang/19/include/"  -Os \
#   -flto \
#   -ffat-lto-objects \
#   -g0 \
#   -c $APP/round_trip_c.c \
#   -o $APP/round_trip_c.o

riscv32-none-elf-gcc -c $APP/round_trip_c.c -Os -mcmodel=medany -march=rv32g -mabi=ilp32d -Wno-implicit-function-declaration -o $APP/round_trip_c.o

cp "$APP/round_trip_c.o"  "$SSD_OS_PATH/app/programs/build_rs/rs_ftl.o"
cp "$APP/connectors.conn" "$SSD_OS_PATH/app/programs/rs_ftl.conn"
cp "$APP/pipelines.pipe" "$SSD_OS_PATH/app/programs/rs_ftl.pipe"
echo "Built app"
