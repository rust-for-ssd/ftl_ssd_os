test:
	cargo qt 

build:
	./compile.sh
	
plot: 
	python3 ./benchmark/plots.py

connector_per_component:
	APP_NAME=connector_per_component ./compile.sh

pipeline_per_cmd:
	APP_NAME=pipeline_per_cmd ./compile.sh

round_trip:
	APP_NAME=round_trip ./compile.sh
	
round_trip_c:
	./src/apps/round_trip_c/compile.sh

distributed_l2p:
	APP_NAME=distributed_l2p ./compile.sh