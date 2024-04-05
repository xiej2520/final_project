#!/bin/bash

SCRIPT_DIR=$(dirname "$0")

# Download the data
OSM_PBF_FILE="us-northeast.osm.pbf"
if [ ! -f "/data/${OSM_PBF_FILENAME}" ]; then
    wget -P /data "https://grading.cse356.compas.cs.stonybrook.edu/data/${OSM_PBF_FILENAME}"
fi

###########################################################################################################

# # Create volumes for tiles
# docker volume create osm-data
# docker volume create osm-tiles

# Import the node data
docker run \
    -p 8081:80 \
    -p 5432:5432 \
    -v /data/${OSM_PBF_FILE}:/data/region.osm.pbf \
    -v osm-data:/data/database/ \
    -v osm-tiles:/data/tiles/ \
    --shm-size="3gb" \
    overv/openstreetmap-tile-server \
    import

###########################################################################################################

# Create volumes for routing
docker volume create osm-routing

# Build the routing service
docker build -t services/routing $SCRIPT_DIR/../services/routing 

# Import the routing data
docker run \
    -e POSTGRES_PASSWORD=postgres \
    -p 5433:5432 \
    -v /data/${OSM_PBF_FILE}:/data/region.osm.pbf \
    -v osm-routing:/var/lib/postgresql/data \
    --shm-size="3gb" \
    services/routing \
    import