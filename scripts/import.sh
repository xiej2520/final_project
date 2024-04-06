#!/bin/bash

SCRIPT_DIR=$(dirname "$0")

function usage() {
    echo "Usage: $0 region"
    echo "  region: The region to import."
}

if [ "$#" -ne 1 ]; then
    usage
    exit 1
fi

# Download the data
REGION=$1
if [ ! -f "/data/${REGION}.osm.pbf" ]; then
    wget -P /data "https://grading.cse356.compas.cs.stonybrook.edu/data/${REGION}.osm.pbf"
fi

# Create volumes for routing
docker volume create osm-data

# Build the routing service
docker build -t import_database $SCRIPT_DIR/../services/database

# Import the routing data
docker run --rm \
    -e POSTGRES_PASSWORD=postgres \
    -v /data/${REGION}.osm.pbf:/data/region.osm.pbf \
    -v osm-data:/var/lib/postgresql/data \
    import_database