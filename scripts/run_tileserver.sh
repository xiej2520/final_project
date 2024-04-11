#!/bin/bash

if [ "$#" -ne 1 ]; then
    echo "Usage: $0 region_mbtiles"
    echo "  region_mbtiles: The mbtiles file for the region to serve."
    exit 1
fi

FILE=$1

docker run --rm --it \
  -v /data/${FILE}:/data/region.mbtiles \
  -p 8080:8080 \
  maptiler/tileserver-gl \
  --file /data/region.mbtiles
