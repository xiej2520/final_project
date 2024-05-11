# Final Project

## Solution

This can be run on one machine, as either a monoservice (one server binary on host)
or split into microservices (separate binaries running in containers). In either
case, they will communicate with tileserver, database, and osrm routing running
in Docker containers.
The project can also be deployed as a distributed service using the ansible
playbook.
In any case, you nedd to import the data into the tileserver, database, and osrm
formats, which can be copied to the distributed nodes instead of importing on
each machine.

### Import data

1. Run `./scripts/import.sh --all --region <region>` to import data
   * Note: osrm requires a lot of memory to create the routing data and will
   crash if there is not enough memory. `us-northeast` required >12.8 GB.
   * Check the usage flags to import data for services individually.

#### Export data

`import.sh` should have created compressed versions of the imported data to be
copied to downstream nodes in the `/data` directory. Check `./scripts/export.sh`
for the commands to manually do this.

### Mono service 

To run the service with a single server binary, useful for testing:

1. Run `./scripts/install_docker.sh` to install docker and docker-volume-snapshot
2. Run `./scripts/install_rust.sh` to install Rust
3. Run `cargo build --release` to build the server
4. `cd` to `./scripts/monoservice` and run `docker compose up -d` to run the
containers.
  Provide the region (`REGION=us-northeast docker compose up -d`) or
  * Create `.env` file if you want (see example)
5. Run the rust server `cargo run --release`.
  Make sure you have a `config.toml` (see example) in the root directory.

### Microservices on one machine

1. Run `./scripts/install_docker.sh` to install docker and docker-volume-snapshot
2. `cd` to `./scripts/microservices_local` and run `docker compose up -d` to run
the containers.
  * Make sure you have a `config.toml` (see example) in the root directory, this
  will be copied into the services' docker images.
  * **Make sure to rebuild the images** (`docker compose up --build`?? or
  delete all images) to see changes in the Rust code or config.

### Distributed service 

1. Run `./scripts/install_rust.sh` to install rust
2. Run `cargo build --release` to build services
3. Run `python3 -m pip install --user ansible` to install ansible
4. Create `inventory.ini` file (see example)
5. Run `ansible-playbook -i inventory.ini -e "REGION=<region> DOMAIN=<domain> SUBMISSION_ID=<submission_id>" playbook.yml`
to deploy using ansible
   * Also include `--private-key {key-file}`

Make sure you have a `inventory.ini` in root directory.
If you'd like you can also create a `extra_vars.yml` file to store the extra vars. 

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
turn_url = "http://tileserver:8080/styles/osm-bright/static/"
route_url = "http://osrm-backend:5000/route/v1/driving/"

# nginx will serve this instead, this is just for testing (needed in config still)
tile_url = "http://tileserver:8080/styles/osm-bright/256/"
# redis cache attempt
# cache_url = "redis://cache:6379"
# monoservice urls
db_url = "postgresql://renderer:renderer@localhost:5432/gis"
cache_url = "redis://localhost:6379"
turn_url = "http://localhost:8081/styles/osm-bright/static/"
route_url = "http://localhost:5000/route/v1/driving/"
```

* Compile with `disable_email` flag to disable sending emails for verification.
* Compile with `disable_auth` flag to disable authentication requirement for
endpoints.
* Compile with `disable_logs` flag to disable tracing logs.

```shell
cargo +nightly run -F disable_email -F disable_auth
```

## dotenv

```env
REGION=monaco-latest

AUTH_PORT=8000
ROUTING_PORT=8001
SEARCH_PORT=8002
TILES_PORT=8003
```

## Inventory

Change the ips to be the servers you're deploying to.

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
