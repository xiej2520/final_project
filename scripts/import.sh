#!/bin/bash

set -euo pipefail

function usage() {
>&2 cat << EOF
Usage: $0
  [ --all ]
  [ --nominatim ]
  [ --tileserver ] 
  [ --osrm-backend ]
  [ --region <region> ]
  [ --url <pbf_url> ]
EOF
exit 1
}

IMPORT_NOMINATIM=0
IMPORT_OSRM_BACKEND=0
IMPORT_TILESERVER=0
REGION=
PBF_URL=

OPTIONS=$(getopt -o "" --long all,help,nominatim,osrm-backend,tileserver,region:,url: -- "$@")
if [[ $? -gt 0 ]]; then
  usage
fi

eval set -- ${OPTIONS}
while :
do
  case $1 in
    --all)
      IMPORT_NOMINATIM=1
      IMPORT_OSRM_BACKEND=1
      IMPORT_TILESERVER=1
      shift
      ;;
    --help)         usage                   ; shift   ;;
    --nominatim)    IMPORT_NOMINATIM=1      ; shift   ;;
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

if [[ $IMPORT_NOMINATIM -eq 1 ]]; then
  IMPORT_FINISHED=/var/lib/postgresql/14/main/import-finished

  docker volume create nominatim-data
  docker volume create nominatim-flatnode

  docker run --rm \
    -v /data:/data \
    -v nominatim-data:/var/lib/postgresql/14/main \
    -v nominatim-flatnode:/nominatim/flatnode \
    -e PBF_PATH="/data/${PBF_FILENAME}" \
    -e FREEZE=true \
    mediagis/nominatim:4.4 \
    /bin/bash -c "/app/config.sh && useradd -m nominatim && /app/init.sh && touch ${IMPORT_FINISHED}"

  docker volume rm nominatim-flatnode # flatnodes unneeded after import

  ##### DO NOT DELETE
  ## import planet_osm_line, planet_osm_roads, planet_osm_polygon, planet_osm_point
  ## for custom search
  ##docker run -i -t --rm openfirmware/osm2pgsql -c ' osm2pgsql -c -d postgres://nominatim:qaIACxO6wMR3@localhost:5432/nominatim /data/${PBF_FILENAME}'

  # import photon
  docker run --rm -d \
    --name nominatim_db \
    -v /data:/data \
    -v nominatim-data:/var/lib/postgresql/14/main \
    -p 5432:5432 \
    -e PBF_PATH="/data/${PBF_FILENAME}" \
    mediagis/nominatim:4.4 
  
  wget -P /data https://github.com/komoot/photon/releases/download/0.5.0/photon-0.5.0.jar

  docker run --net=host \
    -v /data:/data \
    -v nominatim-data:/photon \
    amazoncorretto:22.0.1-alpine3.19 \
    java -jar /data/photon-0.5.0.jar \
    -nominatim-import \
    -host localhost -port 5432 \
    -database nominatim -password qaIACxO6wMR3 \
    -data-dir /photon

  docker stop nominatim_db
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
