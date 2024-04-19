#!/bin/bash

set -e
DIR=$(realpath "$(dirname "${BASH_SOURCE[0]}")")

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

echo "RUNNING " $DIR/import_osrm.sh $REGION
$DIR/import_osrm.sh $REGION

echo "RUNNING " $DIR/import_tiles.sh $REGION
$DIR/import_tiles.sh $REGION

echo "RUNNING " $DIR/import_nominatim.sh $REGION
$DIR/import_nominatim.sh $REGION
