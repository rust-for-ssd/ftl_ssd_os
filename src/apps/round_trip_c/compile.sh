#!/bin/sh
SSD_OS_PATH=${SSD_OS_PATH:-../ssd_os}
APP="src/apps/round_trip_c"
# IDIR="./ssd_os"
# CFLAGS="-Wall -I $IDIR -O3 -mcmodel=medany -Wno-implicit-function-declaration"


# clang -c $APP/round_trip_c.c -o $APP/round_trip_c.o

riscv32-none-elf-gcc -c $APP/round_trip_c.c -O3 -mcmodel=medany -march=rv32g -mabi=ilp32d -Wno-implicit-function-declaration -o $APP/round_trip_c.o

cp "$APP/round_trip_c.o"  "$SSD_OS_PATH/app/programs/build_rs/rs_ftl.o"
cp "$APP/connectors.conn" "$SSD_OS_PATH/app/programs/rs_ftl.conn"
cp "$APP/pipelines.pipe" "$SSD_OS_PATH/app/programs/rs_ftl.pipe"
echo "Built app"
