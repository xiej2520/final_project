#!/bin/bash

set -euo pipefail

function usage() {
>&2 cat << EOF
Usage: $0
  [ --all ]
  [ --database ]
  [ --tileserver ] 
  [ --osrm-backend ]
  [ --region <region> ]
  [ --url <pbf_url> ]
EOF
exit 1
}

IMPORT_DATABASE=0
IMPORT_OSRM_BACKEND=0
IMPORT_TILESERVER=0
REGION=
PBF_URL=

OPTIONS=$(getopt -o "" --long all,help,database,osrm-backend,tileserver,region:,url: -- "$@")
if [[ $? -gt 0 ]]; then
  usage
fi

eval set -- ${OPTIONS}
while :
do
  case $1 in
    --all)
      IMPORT_DATABASE=1
      IMPORT_OSRM_BACKEND=1
      IMPORT_TILESERVER=1
      shift
      ;;
    --help)         usage                   ; shift   ;;
    --database)     IMPORT_DATABASE=1       ; shift   ;;
    --osrm-backend) IMPORT_OSRM_BACKEND=1   ; shift   ;;
    --tileserver)   IMPORT_TILESERVER=1     ; shift   ;;
    --region)       REGION=$2               ; shift 2 ;;
    --url)          PBF_URL=$2              ; shift 2 ;;
    # -- means the end of the arguments; drop this, and break out of the while loop
    --) shift; break ;;
    *) >&2 echo Unsupported option: $1
       usage ;;
  esac
done

if [[ "$REGION" == "" && "$PBF_URL" == "" ]] || [[ "$REGION" != "" && "$PBF_URL" != "" ]]; then
  usage
elif [[ "$REGION" != "" ]]; then
  PBF_FILENAME="${REGION}.osm.pbf"
  PBF_URL="https://grading.cse356.compas.cs.stonybrook.edu/data/${PBF_FILENAME}"
elif [[ "$PBF_URL" != "" ]]; then
  PBF_FILENAME=$(basename "$PBF_URL")
  REGION=${PBF_FILENAME%%.*}
fi

# Download the data
if [[ ! -f "/data/${PBF_FILENAME}" ]]; then
  wget -P /data $PBF_URL
fi

if [[ $IMPORT_DATABASE -eq 1 ]]; then
  docker volume create osm-data

  # OSM2PGSQL_EXTRA_ARGS -C: MB RAM cache
  docker run --rm \
    -v /data/${PBF_FILENAME}:/data/region.osm.pbf \
    -v osm-data:/data/database/ \
    -e "FLAT_NODES=enabled" \
    -e "threads=8" \
    -e "OSM2PGSQL_EXTRA_ARGS=-C 4096" \
    overv/openstreetmap-tile-server \
    import
  
  docker run --rm \
    -v osm-data:/data/database/ \
    busybox \
    rm /data/database/flat_nodes.bin
fi

if [[ $IMPORT_TILESERVER -eq 1 ]]; then
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
  cp assets/tileserver/tileserver_config.json /data/tileserver/config.json
  cp assets/tileserver/styles/* /data/tileserver/styles/
  sed -i "s/zurich_switzerland/${REGION}/g" /data/tileserver/config.json
  rm test_data.zip /data/tileserver/zurich_switzerland.mbtiles
fi

if [[ $IMPORT_OSRM_BACKEND -eq 1 ]]; then
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
fi
