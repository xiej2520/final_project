# Final Project

## Solution

1. Run `./scripts/install_docker.sh` to install docker
2. Run `./scripts/import.sh <region>` to import data
3. Run `REGION=<region> docker compose up -d` to run services
4. Run `./scripts/install_rust.sh` to install rust
5. Run `./scripts/run_tileserver.sh` to run tileserver
6. Run `cargo run --release` to run server

Make sure you have a `config.toml` in the root directory.

### To copy to remote machine

* Run `./scripts/import.sh <region>` to import data on local machine
* Copy `/data` directory to remote machine.
* Use [docker-volume-snapshot](https://github.com/junedkhatri31/docker-volume-snapshot)
to transfer `osm-data` volume to remote machine.
* Run `REGION=<region> docker compose up -d` to run services on remote.

## Config

Example `config.toml`

```toml
ip = [127, 0, 0, 1]
http_port = 80
domain = "not-invented-here.cse356.compas.cs.stonybrook.edu"

# email stuff
relay_ip = [130, 245, 171, 151]
relay_port = 11587

# urls for services (trailing slash significant!)
db_url = "postgresql://renderer:renderer@localhost:5432/gis"
tile_url = "http://localhost:8080/styles/osm-bright/256/"
turn_url = "http://localhost:8080/styles/osm-bright/static/"
routing_url = "http://localhost:5000/route/v1/driving/"

submission_id = "65b54162aa2cfc5a3dea55fe"
```

## Notes

* 4326 is lat/longitude srid
* 3857 is the data's srid
* .way is the geometry column
* `geometry ST_MakeEnvelope(float xmin, float ymin, float xmax, float ymax, integer srid=unknown)`

* transform lat/lon bbox to srid 3857 bbox,
* find polygons intersecting the bbox
* get name, centroid of intersection, transform Y and X back to lat and lon
