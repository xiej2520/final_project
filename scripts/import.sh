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

###########################################################################################################

# # Create volumes for tiles
# docker volume create osm-data
# docker volume create osm-tiles

# # Import the node data
# docker run \
#     -p 8081:80 \
#     -p 5432:5432 \
#     -v /data/${OSM_PBF_FILE}:/data/region.osm.pbf \
#     -v osm-data:/data/database/ \
#     -v osm-tiles:/data/tiles/ \
#     --shm-size="3gb" \
#     overv/openstreetmap-tile-server \
#     import

###########################################################################################################

# Create volumes for routing
docker volume create osm-routing

# Build the routing service
docker build -t import_routing $SCRIPT_DIR/../services/routing 

# Import the routing data
docker run --rm \
    -e POSTGRES_PASSWORD=postgres \
    -v /data/${REGION}.osm.pbf:/data/region.osm.pbf \
    -v osm-routing:/var/lib/postgresql/data \
    import_routing