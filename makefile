test:
	cargo qt 

build:
	./compile.sh
	
plot: 
	python3 ./benchmark/plots.py

connector_per_component:
	APP_NAME=connector_per_component ./compile.sh

