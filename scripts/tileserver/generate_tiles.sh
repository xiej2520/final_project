#!/bin/bash
export DEBIAN_FRONTEND=noninteractive

# install java
sudo apt install openjdk-18-jre

### Generate mbtiles with planetiler
wget https://github.com/onthegomap/planetiler/releases/latest/download/planetiler.jar
java -Xmx4g -jar planetiler.jar --download --osm-path=/data/us-northeast.osm.pbf --output=/data/us-ne.mbtiles
