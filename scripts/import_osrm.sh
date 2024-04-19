#!/bin/bash

set -e

if [ "$#" -ne 1 ]; then
  echo "Usage: $0 pbf_url"
  echo "  pbf_url: The url of the pbf file to download, or region name."
    exit 1
fi

if [[ $1 == http?(s)://*.osm.pbf ]];
then
  # URL parameter
  PBF_URL=$1
  PBF_FILENAME=$(basename "$1")
  REGION=${PBF_FILENAME%%.*}
else
  # region parameter
  REGION=$1
  PBF_FILENAME="$REGION.osm.pbf"
  PBF_URL="https://grading.cse356.compas.cs.stonybrook.edu/data/$PBF_FILENAME"
fi

# Download the data
if [ ! -f "/data/${PBF_FILENAME}" ]; then
  wget -P /data $PBF_URL
fi

mkdir -p /data/osrm
# us-northeast 12.8GB RAM
docker run --rm \
    -t -v /data/osrm:/data \
    -v /data/${PBF_FILENAME}:/data/${PBF_FILENAME} \
    ghcr.io/project-osrm/osrm-backend \
    osrm-extract -p /opt/car.lua \
    /data/${PBF_FILENAME} || echo "osrm-extract failed"

# us-northeast 6.4GB RAM
docker run --rm \
    -t -v /data/osrm:/data \
    -v /data/${PBF_FILENAME}:/data/${PBF_FILENAME} \
    ghcr.io/project-osrm/osrm-backend \
    osrm-partition \
    /data/${REGION}.osrm || echo "osrm-partition failed"

# us-northeast 5.2GB RAM
docker run --rm \
    -t -v /data/osrm:/data \
    -v /data/${PBF_FILENAME}:/data/${PBF_FILENAME} \
    ghcr.io/project-osrm/osrm-backend \
    osrm-customize \
    /data/${REGION}.osrm || echo "osrm-customize failed"
