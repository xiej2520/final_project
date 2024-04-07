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
sudo -u postgres createuser carto
sudo -u postgres createdb -E UTF8 -O carto gis 
sudo -u postgres psql -d gis -c "CREATE EXTENSION hstore;"
sudo -u postgres psql -d gis -c "CREATE EXTENSION postgis;"
sudo -u postgres psql -d gis -c "CREATE EXTENSION postgis_raster;"
sudo -u postgres psql -d gis -c "CREATE EXTENSION pgRouting;"
sudo -u postgres psql -d gis -c "ALTER TABLE spatial_ref_sys OWNER TO carto;"
sudo -u postgres psql -c "ALTER USER carto PASSWORD '${PGPASSWORD:-carto}'"

# Import osm data
osm2pgsql -d gis -U carto --create --slim -G --hstore --number-processes ${THREADS:-4} /data/region.osm.pbf

# # Create tables for routing
# sudo -u postgres createuser router
# sudo -u postgres createdb -E UTF8 -O router routing 
# sudo -u postgres psql -d routing -c "CREATE EXTENSION postgis;"
# sudo -u postgres psql -d routing -c "CREATE EXTENSION pgRouting;"
# sudo -u postgres psql -d routing -c "ALTER TABLE spatial_ref_sys OWNER TO router;"
# sudo -u postgres psql -c "ALTER USER router PASSWORD '${PGPASSWORD_ROUTER:-router}'"

# Download osm2po (http://osm2po.de/)
wget https://osm2po.de/releases/osm2po-5.5.11.zip
unzip -o osm2po-5.5.11.zip
echo "postp.0.class = de.cm.osm2po.plugins.postp.PgRoutingWriter" >> osm2po.config
echo "wtr.finalMask = car,foot,bike" >> osm2po.config

# Generate topology
java -jar osm2po-core-5.5.11-signed.jar cmd=c /data/region.osm.pbf 

# Import routing data
sudo -u postgres psql -U carto -d gis -q -f osm/osm_2po_4pgr.sql