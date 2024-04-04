#!/bin/bash

function setPostgresPassword() {
    sudo -u postgres psql -c "ALTER USER router PASSWORD '${PGRPASSWORD:-router}'"
}

if [ "$1" == "import" ]; then
    # Ensure that database directory is in right state
    mkdir -p /data/database/postgres/
    chown renderer: /data/database/
    chown -R postgres: /var/lib/postgresql /data/database/postgres/
    if [ ! -f /data/database/postgres/PG_VERSION ]; then
        sudo -u postgres /usr/lib/postgresql/$PG_VERSION/bin/pg_ctl -D /data/database/postgres/ initdb -o "--locale C.UTF-8"
    fi

    # Initialize PostgreSQL
    service postgresql start
    sudo -u postgres createuser router
    sudo -u postgres createdb -E UTF8 -O router routing 
    sudo -u postgres psql -d routing -c "CREATE EXTENSION postgis;"
    sudo -u postgres psql -d routing -c "CREATE EXTENSION pgrouting;"
    sudo -u postgres psql -d routing -c "ALTER TABLE spatial_ref_sys OWNER TO router;"
    setPostgresPassword

    # Convert OSM to XML format
    osmconvert /data/region.osm.pbf \ 
        --drop-author \ 
        --drop-version \ 
        --out-osm -o=/data/region.osm

    # Import routing data
    osm2pgrouting --f /data/region.osm \ 
        --dbname routing \ 
        --username router \ 
        --clean    

    # Cleanup
    rm -rf /data/region.osm
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