#!/bin/sh
SSD_OS_PATH=${SSD_OS_PATH:-../ssd_os}
APP="src/apps/round_trip_c"

clang -c $APP/round_trip_c.c -o $APP/round_trip_c.o

cp "$APP/round_trip_c.o"  "$SSD_OS_PATH/app/programs/build_rs/rs_ftl.o"
cp "$APP/connectors.conn" "$SSD_OS_PATH/app/programs/rs_ftl.conn"
cp "$APP/pipelines.pipe" "$SSD_OS_PATH/app/programs/rs_ftl.pipe"
echo "Built app"
exit 1
