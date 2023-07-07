#!/bin/bash

USAGE="Usage: $0 [ build | local | remote ]"

# Function to build docker image
function build_docker_image {
    docker build -t dataset-manager:latest .
}

# Function to run container with dataset in local folder
function run_local_dataset {
    docker run -it --rm \
        -v $(pwd)/clothing-dataset-small:/clothing-dataset-small:ro \
        -v $(pwd)/data:/data:rw \
        -u $(id -u):$(id -g) \
        dataset-manager:latest \
        dataset-manager -o clothing-dataset-small -d data
}

# Function to run container with dataset downloaded from repository
function run_downloaded_dataset {
    docker run -it --rm \
        -v $(pwd)/data:/data:rw \
        -u $(id -u):$(id -g) \
        dataset-manager:latest \
        dataset-manager -d data
}

if [ "$#" -ne 1 ]; then
    echo "Error: invalid number of parameters. $USAGE"
    exit 1
fi

# Check for command line arguments
if [ "$1" == "build" ]; then
    build_docker_image
elif [ "$1" == "local" ]; then
    run_local_dataset
elif [ "$1" == "remote" ]; then
    run_downloaded_dataset
else
    echo "Error: invalid option. $USAGE"
    exit 1
fi
