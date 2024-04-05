#!/bin/bash
export DEBIAN_FRONTEND=noninteractive

# Install postgres
# Create the file repository configuration:
sudo sh -c 'echo "deb https://apt.postgresql.org/pub/repos/apt $(lsb_release -cs)-pgdg main" > /etc/apt/sources.list.d/pgdg.list'

# Import the repository signing key:
wget --quiet -O - https://www.postgresql.org/media/keys/ACCC4CF8.asc | sudo apt-key add -

# Update the package lists:
sudo apt-get update

# Install the latest version of PostgreSQL.
# If you want a specific version, use 'postgresql-12' or similar instead of 'postgresql':
sudo apt-get -y install postgresql
################################################################################

sudo apt install osm2pgsql

# Start PostGIS
docker run --name some-postgis -e POSTGRES_PASSWORD=mysecretpassword -d -p 5432:5432 postgis/postgis

docker exec --user postgres some-postgis psql -c "CREATE DATABASE osm;"
docker exec --user postgres some-postgis psql osm -c "CREATE EXTENSION postgis;"
docker exec --user postgres some-postgis psql osm -c "CREATE EXTENSION postgis_raster;"
#docker exec -ti some-postgis psql -U postgres
osm2pgsql -c -d osm -W -H 127.0.0.1 -P 5432 -U postgres /data/us-northeast.osm.pbf
# enter password 'mysecretpassword'
osm2pgsql -c postgresql://postgres:mysecretpassword@127.0.0.1:5432/osm /data/us-northeast.osm.pbf

sudo apt install libmapnik-dev mapnik-utils python3-mapnik

#git clone https://github.com/gravitystorm/openstreetmap-carto
#cd openstreetmap-carto

# maplibre native
git clone --recurse-submodules -j8 https://github.com/maplibre/maplibre-native.git
cd maplibre-native
docker build -t maplibre-native-image .

cargo install martin