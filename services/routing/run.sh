#!/bin/bash

set -euo pipefail

# Start PostgreSQL
DB_DIR=/var/lib/postgresql/data
mkdir -p $DB_DIR 
chown -R postgres: /var/lib/postgresql
if [ ! -f $DB_DIR/PG_VERSION ]; then
    sudo -u postgres /usr/lib/postgresql/$PG_VERSION/bin/pg_ctl -D $DB_DIR initdb -o "--locale C.UTF-8"
fi
sudo -u postgres /usr/lib/postgresql/$PG_VERSION/bin/pg_ctl -D $DB_DIR start

# Create tables
sudo -u postgres createuser router
sudo -u postgres createdb -E UTF8 -O router routing 
sudo -u postgres psql -d routing -c "CREATE EXTENSION postgis;"
sudo -u postgres psql -d routing -c "CREATE EXTENSION pgRouting;"
sudo -u postgres psql -d routing -c "ALTER TABLE spatial_ref_sys OWNER TO router;"
sudo -u postgres psql -c "ALTER USER router PASSWORD '${PGPASSWORD:-router}'"

# Download osm2po (http://osm2po.de/)
wget https://osm2po.de/releases/osm2po-5.5.11.zip
unzip -n osm2po-5.5.11.zip

# Generate topology
java -jar osm2po-core-5.5.11-signed.jar cmd=c /data/region.osm.pbf 

# Import data
sudo -u postgres psql -U router -d routing -q -f osm/osm_2po_4pgr.sql