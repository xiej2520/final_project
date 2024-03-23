#!/bin/bash

sudo apt update

# Install docker
sudo apt-get install ca-certificates curl -y
sudo install -m 0755 -d /etc/apt/keyrings
sudo curl -fsSL https://download.docker.com/linux/ubuntu/gpg -o /etc/apt/keyrings/docker.asc
sudo chmod a+r /etc/apt/keyrings/docker.asc

# Add the repository to Apt sources:
echo \
  "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/ubuntu \
  $(. /etc/os-release && echo "$VERSION_CODENAME") stable" | \
  sudo tee /etc/apt/sources.list.d/docker.list > /dev/null
sudo apt-get update

sudo apt-get install docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin -y

sudo service docker start

# Install rust
sudo apt install build-essential libssl-dev pkg-config

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

rustup toolchain install nightly
rustup default nightly
rustup target add x86_64-unknown-linux-musl
rustup component add rustc-codegen-cranelift-preview --toolchain nightly
MOLD_RELEASE=https://github.com/rui314/mold/releases/download/v2.30.0/mold-2.30.0-x86_64-linux.tar.gz
wget -O - "$MOLD_RELEASE" | sudo tar -C /usr/local --strip-components=1 --no-overwrite-dir -xzf -

# cargo +nightly build --profile=fast-dev-linux
cargo build

wget -P static https://grading.cse356.compas.cs.stonybrook.edu/data/new-york.osm.pbf
sudo apt install osm2pgsql

# Start PostGIS
docker run --name some-postgis -e POSTGRES_PASSWORD=mysecretpassword -d -p 5432:5432 postgis/postgis

docker exec --user postgres some-postgis psql -c "CREATE DATABASE osm;"
docker exec --user postgres some-postgis psql osm -c "CREATE EXTENSION postgis;"
docker exec --user postgres some-postgis psql osm -c "CREATE EXTENSION postgis_raster;"
#docker exec -ti some-postgis psql -U postgres
#osm2pgsql -c -d osm -W -H 127.0.0.1 -P 5432 -U postgres static/new-york.osm.pbf
# enter password 'mysecretpassword'
osm2pgsql -c postgresql://postgres:mysecretpassword@127.0.0.1:5432/osm static/new-york.osm.pbf

#sudo apt install libmapnik-dev mapnik-utils python3-mapnik
#
#git clone https://github.com/gravitystorm/openstreetmap-carto
#cd openstreetmap-carto
#

# cargo install martin


mkdir /data
cp static/new-york.osm.pbf /data/new-york.osm.pbf

docker run \
    -v /data/new-york.osm.pbf:/data/region.osm.pbf \
    -v osm-data:/data/database/ \
    overv/openstreetmap-tile-server \
    import

docker run \
    -p 8080:80 \
    -v osm-data:/data/database/ \
    -d overv/openstreetmap-tile-server \
    run
