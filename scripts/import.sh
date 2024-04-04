#!/bin/bash

# Download the data
OSM_PBF_FILE="us-northeast.osm.pbf"
wget -P /data "https://grading.cse356.compas.cs.stonybrook.edu/data/${OSM_PBF_FILE}"

# Create the volumes
docker volume create osm-data
docker volume create osm-tiles

# Import the node data
docker run \
    -p 8080:80 \
    -p 5432:5432 \
    -v /data/${OSM_PBF_FILE}:/data/region.osm.pbf \
    -v osm-data:/data/database/ \
    -v osm-tiles:/data/tiles/ \
    --shm-size="3gb" \
    overv/openstreetmap-tile-server \
    import

# Build the routing service
docker build -t services/routing ../services/routing 

# Import the routing data
docker run \
    -p 5433:5432 \
    -v /data/${OSM_PBF_FILE}:/data/region.osm.pbf \
    -v osm-data:/data/database \
    --shm-size="3gb" \
    services/routing \
    import