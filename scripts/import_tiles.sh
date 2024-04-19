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

# Create plane tiles
mkdir -p /data/tiles
docker run --rm \
  -e JAVA_TOOL_OPTIONS="-Xmx2g" \
  -v /data/tiles:/data \
  -v /data/${PBF_FILENAME}:/data/${PBF_FILENAME} \
  ghcr.io/onthegomap/planetiler --download \
  --osm-path=/data/${PBF_FILENAME} \
  --output=/data/${REGION}.mbtiles

# set up tileserver config
mkdir -p /data/tileserver
wget https://github.com/maptiler/tileserver-gl/releases/download/v1.3.0/test_data.zip
unzip -o test_data.zip -d /data/tileserver
cp static/tileserver_config.json /data/tileserver/config.json
sed -i "s/zurich_switzerland/${REGION}/g" /data/tileserver/config.json
rm test_data.zip /data/tileserver/zurich_switzerland.mbtiles
