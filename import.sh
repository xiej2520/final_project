#!/bin/bash

# Download the data
wget -P /data "https://grading.cse356.compas.cs.stonybrook.edu/data/new-york.osm.pbf"

# Create the volumes
docker volume create osm-data
docker volume create osm-tiles

# Import the data
docker run \
    -p 8080:80 \
    -p 5432:5432 \
    -v /data/new-york.osm.pbf:/data/region.osm.pbf \
    -v osm-data:/data/database/ \
    -v osm-tiles:/data/tiles/ \
    --shm-size="3gb" \
    overv/openstreetmap-tile-server \
    import