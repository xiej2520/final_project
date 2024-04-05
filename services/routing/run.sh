#!/bin/bash

set -euo pipefail

function setPostgresPassword() {
    sudo -u postgres psql -c "ALTER USER router PASSWORD '${PGRPASSWORD:-router}'"
}

if [ "$1" == "import" ]; then
    # Initialize PostgreSQL
    service postgresql start
    sudo -u postgres createuser router
    sudo -u postgres createdb -E UTF8 -O router routing 
    sudo -u postgres psql -d routing -c "CREATE EXTENSION postgis;"
    sudo -u postgres psql -d routing -c "CREATE EXTENSION pgrouting;"
    sudo -u postgres psql -d routing -c "ALTER TABLE spatial_ref_sys OWNER TO router;"
    setPostgresPassword

    # Download osm2po (http://osm2po.de/)
    wget -P /tmp/osm2po https://osm2po.de/releases/osm2po-5.5.11.zip
    unzip -o /tmp/osm2po/osm2po-5.5.11.zip -d /tmp/osm2po 
    cp /osm2po.config /tmp/osm2po/osm2po.config

    # Generate topology
    java -jar /tmp/osm2po/osm2po-core-5.5.11-signed.jar cmd=c /data/region.osm.pbf 

    # Import data
    psql -U router -d routing -q -f /tmp/osm2po/osm2po_4pgr.sql

    # Cleanup
    rm -rf /tmp/osm2po
    service postgresql stop
    exit 0
fi

if [ "$1" == "run" ]; then
    # Start PostgreSQL
    service postgresql start
    setPostgresPassword
        
    # Run while handling docker stop's SIGTERM
    stop_handler() {
        kill -TERM "$child"
    }
    trap stop_handler SIGTERM

    sudo -u renderer renderd -f -c /etc/renderd.conf &
    child=$!
    wait "$child"

    service postgresql stop

    exit 0
fi