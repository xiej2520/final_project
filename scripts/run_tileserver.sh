#!/bin/bash

if [ "$#" -ne 1 ]; then
    echo "Usage: $0 region_mbtiles"
    echo "  region_mbtiles: The mbtiles file for the region to serve."
    exit 1
fi

FILE=$1

wget https://github.com/maptiler/tileserver-gl/releases/download/v1.3.0/test_data.zip
sudo unzip test_data.zip -d /data/tileserver-gl

docker run --rm -it \
  -v /data/${FILE}:/data/region.mbtiles \
  -v /data/tileserver-gl:/data \
  -p 8080:8080 \
  maptiler/tileserver-gl \
  --file /data/region.mbtiles
