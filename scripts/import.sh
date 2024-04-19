#!/bin/bash

if [ "$#" -ne 1 ]; then
  echo "Usage: $0 pbf_url"
  echo "  pbf_url: The url of the pbf file to download."
  exit 1
fi

PBF_URL=$1
PBF_NAME=$(basename "$1")
REGION=${PBF_NAME%%.*}

# Download the data
if [ ! -f "/data/${PBF_NAME}" ]; then
  wget -P /data "${PBF_URL}"
fi

# Import nominatim data
IMPORT_FINISHED=/var/lib/postgresql/14/main/import-finished

docker volume create nominatim-data
docker volume create nominatim-flatnode

docker run --rm \
  -v /data:/data \
  -v nominatim-data:/var/lib/postgresql/14/main \
  -v nominatim-flatnode:/nominatim/flatnode \
  -e PBF_PATH="/data/${PBF_NAME}" \
  mediagis/nominatim:4.4 \
  /bin/bash -c "/app/config.sh && useradd -m nominatim && /app/init.sh && touch ${IMPORT_FINISHED}"

# Create plane tiles
docker run --rm \
  -e JAVA_TOOL_OPTIONS="-Xmx2g" \
  -v /data:/data \
  ghcr.io/onthegomap/planetiler --download \
  --osm-path=/data/${PBF_NAME} \
  --output=/data/${REGION}.mbtiles

wget https://github.com/maptiler/tileserver-gl/releases/download/v1.3.0/test_data.zip
unzip -o test_data.zip -d /data
cp static/tileserver_config.json /data/config.json
sed -i "s/zurich_switzerland/${REGION}/g" /data/config.json
rm test_data.zip /data/zurich_switzerland.mbtiles

# us-northeast 12.8GB RAM
docker run --rm \
  -t -v /data:/data \
  ghcr.io/project-osrm/osrm-backend \
  osrm-extract -p /opt/car.lua \
  /data/${PBF_NAME} || echo "osrm-extract failed"

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
