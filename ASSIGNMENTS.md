# Final Project

## Milestone #1

Description

Implement an interactive map viewer, including navigation (panning/scrolling)
and the ability to search for the named objects.  Search should be by object
name (e.g., address, street name, town name, etc.).  Search responses should be
in the order of distance from the center of the query bounding box.

The UI should show the results of the search.  If the search was requested with
onlyInBox true, then all results are within the current bounding box and they
should all be labeled with pins on top of the map without changing the view.
If the search was requested with onlyInBox false, then if there is only one
result, the view should change to show that result in the center of the view,
and if there are multiple results, the UI should show the list of results and
allow the user to click on one of them to change the view such that the clicked
item is shown.

Your server must serve map tiles and handle search queries. Use open-source
tools for map tile generation and spatial databases for GIS data.  Download the
data for this assignment from here:
https://grading.cse356.compas.cs.stonybrook.edu/data/new-york.osm.pbf

### API Endpoints:

* Map Tiles Endpoint: http://your.server/tiles/$LAYER/$V/$H.png
  * Where $LAYER, $V, and $H represent the zoom level, vertical, and horizontal
  tile indices, respectively. 
  * Note: We are forcing the tiles to be from left to right, up to down. If a
  tile x is on the right of a tile y, then $V of x is greater than $V of y.
  * Request Type: GET
  * Response: PNG image of the requested map tile.

* Search Endpoint: http://your.server/api/search
  * Request Type: POST
  * Request Body (JSON):
{
  "bbox": {
    "minLat": number,
    "minLon": number,
    "maxLat": number,
    "maxLon": number
  },
  "onlyInBox": boolean,
  "searchTerm": string
}
  * Response (JSON): A list of objects within the bounding box matching the
  search term. Each object includes the name and coordinates.  If onlyInBox is
  true, then only objects within the query bbox are returned, with coordinates
  pointing to the center of the VISIBLE PORTION of the object within the queried
  bounding box.  If onlyInBox is false, then coordinates are the center of the
  object and bbox is the bounding box that includes the entire object.
[
  {
    "name": "string",
    "coordinates": {
      "lat": number,
      "lon": number
    }
    "bbox": {
      "minLat": number,
      "minLon": number,
      "maxLat": number,
      "maxLon": number
    },
  }
]

* Convert: http://your.server/convert
  * Request Type: POST
  * Request Body (JSON): given zoom level, latitude and longitude of a point,
  convert to x y coordinate ($V $H) of tile indices that contains the point in
  Map Tiles Endpoint
  e.g: if the zoom/lat/long of Manhattan is passed, return index of the tile
  that contains Manhattan.
{
  "lat": number,
  "long": number,
  "zoom": number
}
Response(JSON): 
{
  "x_tile": number,
  "y_tile": number
}


Remember to add x-cse356 header.
HINT: Look up open-source databases capable of storing and querying GIS data.
HINT: For generating and serving map tiles, explore open-source tools mentioned
on the OpenStreetMap wiki.

## Milestone 2

Description

Extend the interactive map viewer to include the ability for users to create
accounts and log in.  Users who are logged in should now have the option of
using the route finding functionality. Implement an API (accessible only to
logged in users) that accepts source and destination locations, and returns a
JSON list where each element represents the next step in the turn-by-turn
directions for the requested route.

Update the UI to allow users to input source and destination to find a route
between them.  The system should then display turn-by-turn directions with a
zoomed-in map view at the coordinates of each step.

Download the data for this assignment from here:
  https://grading.cse356.compas.cs.stonybrook.edu/data/us-northeast.osm.pbf

Retain all tile generation and search functionality from Milestone #1.

User management API endpoints:

* /api/adduser { username, password, email }
  * Creates a disabled user that cannot log in (until /api/verify is used).
  * Returns { status: 'ok' } when successful.
* /api/login { username, password }
  * Returns { status: 'ok' } when successful.
* /api/logout { } 
  * Returns { status: 'ok' } when successful.
* /api/user {}
  * Returns { loggedin: true|false, username: ... } to indicate if a user is
  logged in and provides the username if yes.

* GET /api/verify { email, key }
  * Verification link (make sure to include the full link with parameters as
  part of the plain email text) with the two parameters in the query string is
  sent by email. Do not use a third-party mail service (e.g., gmail) for your
  mail server.

* GET /turn/$TL/$BR.png
  * Return an image/png of a zoomed-in map view of a turn, where $TL and $BR
  indicate the top left and bottom right coordinates in the form lat,lon.
  The generated image should be 100x100 pixels.

* Route Endpoint: http://your.server/api/route

Request Body (JSON):
{
  "source": {
    "lat": number,
    "lon": number
  },
  "destination": {
    "lat": number,
    "lon": number
  }
}
Response (JSON): A list representing the route from source to destination.
[
  {
    "description": string,
    "coordinates": {
      "lat": number,
      "lon": number
    }
  },
  ...
]

Hint: Use code and iptables email server rules from Warm-up Project #2.

## Milestone 3

Everything from Milestone #2, plus the following POST route:

Route Endpoint: http://your.server/api/address
Request Body (JSON):
{
  "lat": number,
  "lon": number
}
Response (JSON): The address of the building at the requested location
{
  "number": string,
  "street": string,
  "city": string,
  "state": string,
  "country": string,
}

Target: 1600 requests per second, 95% responding in under 50ms.
(Tiles only, we passed with 178ms 95% @ 1601RPS)
