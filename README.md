# Final Project

## Solution

There are two ways to run as a mono service or distributed service. For both you must import data first!

### Import data

1. Run `./scripts/import.sh --all --region <region>` to import data
   * Note: osrm requires a lot of memory to create the routing data and will
   crash if there is not enough memory

### Mono service 

1. Run `./scripts/install_docker.sh` to install docker and docker-volume-snapshot
2. Run `./scripts/install_rust.sh` to install rust
3. Run `cargo build --release` to build services
4. Create `.env` file (see example)
4. Run `docker compose up -d` to run the containers 

### Distributed service 

1. Run `python3 -m pip install --user ansible` to install ansible
2. Create `.env` file (see example)
3. Create `inventory.ini` file (see example)
4. Run `ansible-playbook -i inventory.ini playbook.yml` to deploy using ansible

Make sure you have a `config.toml`, `.env` and `inventory.ini` (for ansible) in root directory. Make sure all the ports match up.

### To copy to remote machine

* Run `./scripts/import.sh <region>` to import data on local machine
* Copy `/data` directory to remote machine.
* Use [docker-volume-snapshot](https://github.com/junedkhatri31/docker-volume-snapshot)
to transfer `osm-data` volume to remote machine.
* Run `REGION=<region> docker compose up -d` to run services on remote.

## Config

Example `config.toml`

```toml
ip = [0, 0, 0, 0]
http_port = 80
domain = "not-invented-here.cse356.compas.cs.stonybrook.edu"

# email stuff
relay_ip = [130, 245, 171, 151]
relay_port = 11587

# urls for services (trailing slash significant!)
db_url = "postgres://renderer:renderer@database:5432/gis"
tile_url = "http://tileserver:8080/styles/osm-bright/256/"
turn_url = "http://tileserver:8080/styles/osm-bright/static/"
route_url = "http://osrm-backend:5000/route/v1/driving/"
```

* Compile with `disable_email` flag to disable sending emails for verification.
* Compile with `disable_auth` flag to disable authentication requirement for
endpoints.
* Compile with `disable_logs` flag to disable tracing logs.

```shell
cargo +nightly run -F disable_email -F disable_auth
```

## dotenv

```
REGION=monaco-latest

AUTH_PORT=8000
ROUTING_PORT=8001
SEARCH_PORT=8002
TILES_PORT=8003
```

## Inventory

```ini
nginx ansible_user=root ansible_host=0.0.0.0

[auth]
auth01 ansible_user=root ansible_host=0.0.0.0

[routing]
routing01 ansible_user=root ansible_host=0.0.0.0

[search]
search01 ansible_user=root ansible_host=0.0.0.0

[tiles]
tiles01 ansible_user=root ansible_host=0.0.0.0

[auth:vars]
http_port=8000

[routing:vars]
http_port=8001

[search:vars]
http_port=8002

[tiles:vars]
http_port=8003
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
