# Final Project

## Solution

1. Run `./scripts/install_docker.sh` to install docker and docker-volume-snapshot
2. Run `./scripts/import.sh <region>` to import data
   * Note: osrm requires a lot of memory to create the routing data and will
   crash if there is not enough memory
3. Run `REGION=<region> docker compose up -d` to run services
4. Run `./scripts/install_rust.sh` to install rust
5. Run `cargo run --release` to run server

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
submission_id = "FIX THIS"
ip = [127, 0, 0, 1]
http_port = 8000

# email stuff
domain = "not-invented-here.cse356.compas.cs.stonybrook.edu"
relay_ip = [130, 245, 171, 151]
relay_port = 11587

# urls for services (trailing slash significant!)
search_url = "http://localhost:8080"
turn_url = "http://localhost:8081/styles/osm-bright/static/"
routing_url = "http://localhost:5000/route/v1/driving/"

# nginx should be serving this instead, leave it in for testing?
tile_url = "http://localhost:8081/styles/osm-bright/256/"
```

* Compile with `disable_email` flag to disable sending emails for verification.
* Compile with `disable_auth` flag to disable authentication requirement for
endpoints.
* Compile with `disable_logs` flag to disable tracing logs.

```Shell
cargo +nightly run -F disable_email -F disable_auth
```

## Notes

* 4326 is lat/longitude srid
* 3857 is the data's srid
* .way is the geometry column
* `geometry ST_MakeEnvelope(float xmin, float ymin, float xmax, float ymax, integer srid=unknown)`

* transform lat/lon bbox to srid 3857 bbox,
* find polygons intersecting the bbox
* get name, centroid of intersection, transform Y and X back to lat and lon

* The tileserver-gl style.json changes:
  * `sources.openmaptiles.url` changed to `mbtiles://{openmaptiles}`
  * `glyphs` changed to `{fontstack}/{range}.pbf`
