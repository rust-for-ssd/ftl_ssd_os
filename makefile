test:
	cargo qt 

build:
	./compile.sh
	
plot: 
	python3 ./benchmark/plots.py

connector_per_component:
	APP_NAME=connector_per_component ./compile.sh

round_trip:
	APP_NAME=round_trip ./compile.sh
	
round_trip_c:
	./src/apps/round_trip_c/compile.sh
