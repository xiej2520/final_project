#!/bin/bash

if [ "$#" -ne 1 ]; then
    echo "Usage: $0 region"
    echo "  region: The region to import."
    exit 1
fi

REGION=$1

# Download the data
if [ ! -f "/data/${REGION}.osm.pbf" ]; then
    wget -P /data "https://grading.cse356.compas.cs.stonybrook.edu/data/${REGION}.osm.pbf"
fi

docker volume create osm-data

docker run --rm \
    -v /data/${REGION}.osm.pbf:/data/region.osm.pbf \
    -v osm-data:/data/database \
    -e THREADS=8 \
    -e "FLAT_NODES=enabled" \
    --shm-size="4gb" \
    overv/openstreetmap-tile-server \
    import

# Create plane tiles
docker run --rm \
    -e JAVA_TOOL_OPTIONS="-Xmx2g" \
    -v /data:/data \
    ghcr.io/onthegomap/planetiler --download \
    --osm-path=/data/${REGION}.osm.pbf \
    --output=/data/${REGION}.mbtiles

# us-northeast 12.8GB RAM
docker run --rm \
    -t -v /data:/data \
    ghcr.io/project-osrm/osrm-backend \
    osrm-extract -p /opt/car.lua \
    /data/${REGION}.osm.pbf || echo "osrm-extract failed"

# us-northeast 6.4GB RAM
docker run --rm \
    -t -v /data:/data \
    ghcr.io/project-osrm/osrm-backend \
    osrm-partition \
    /data/${REGION}.osrm || echo "osrm-partition failed"

# us-northeast 5.2GB RAM
docker run --rm \
    -t -v /data:/data \
    ghcr.io/project-osrm/osrm-backend \
    osrm-customize \
    /data/${REGION}.osrm || echo "osrm-customize failed"
