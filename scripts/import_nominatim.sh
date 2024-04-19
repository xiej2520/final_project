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

# Import nominatim data
IMPORT_FINISHED=/var/lib/postgresql/14/main/import-finished

docker volume create nominatim-data
docker volume create nominatim-flatnode

docker run --rm \
  -v /data:/data \
  -v nominatim-data:/var/lib/postgresql/14/main \
  -v nominatim-flatnode:/nominatim/flatnode \
  -e PBF_PATH="/data/${PBF_FILENAME}" \
  -e FREEZE=true \
  --shm-size=8g \
  mediagis/nominatim:4.4 \
  /bin/bash -c "/app/config.sh && useradd -m nominatim && /app/init.sh && touch ${IMPORT_FINISHED}"

docker volume rm nominatim-flatnode # flatnodes unneeded after import
