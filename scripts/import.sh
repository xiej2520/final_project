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

  docker run --rm \
    -v /data/${PBF_FILENAME}:/data/region.osm.pbf \
    -v osm-data:/data/database/ \
    -e "THREADS=$(nproc)" \
    -e "OSM2PGSQL_EXTRA_ARGS=--cache 4096 --drop" \
    overv/openstreetmap-tile-server \
    import

  container_id=$(
    docker run --rm \
      -dp 5432:5432 \
      -v osm-data:/data/database/ \
      overv/openstreetmap-tile-server \
      run
  )

  function check_postgres_ready() {
    pg_isready -h localhost >/dev/null 2>&1
  }

  echo "Waiting for PostgreSQL to become ready..."
  while ! check_postgres_ready; do
    sleep 1
  done

  docker exec $container_id \
    sudo -u postgres psql -d gis -c "CREATE EXTENSION pg_trgm;"
  docker exec $container_id \
    sudo -u postgres psql -d gis -c \
    "CREATE INDEX planet_osm_line_name_idx ON planet_osm_line USING gin(name gin_trgm_ops);"
  docker exec $container_id \
    sudo -u postgres psql -d gis -c \
    "CREATE INDEX planet_osm_point_name_idx ON planet_osm_point USING gin(name gin_trgm_ops);"
  docker exec $container_id \
    sudo -u postgres psql -d gis -c \
    "CREATE INDEX planet_osm_polygon_name_idx ON planet_osm_polygon USING gin(name gin_trgm_ops);"
  docker exec $container_id \
    sudo -u postgres psql -d gis -c \
    "CREATE INDEX planet_osm_roads_name_idx ON planet_osm_roads USING gin(name gin_trgm_ops);"

  docker stop $container_id
  docker rm $container_id

  docker-volume-snapshot create osm-data /data/osm-data.tar
  zstd /data/osm-data.tar 
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
