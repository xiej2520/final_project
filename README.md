# Milestone #1

## Description

Implement an interactive map viewer, including navigation (panning/scrolling)
and the ability to search for the named objects.  Search should be by object
name (e.g., address, street name, town name, etc.).  Search responses should be
in the order of distance from the center of the query bounding box.

The UI should show the results of the search.  If the search was requested with
onlyInBox true, then all results are within the current bounding box and they
should all be labeled with pins on top of the map without changing the view.  If
the search was requested with onlyInBox false, then if there is only one result,
the view should change to show that result in the center of the view, and if there
are multiple results, the UI should show the list of results and allow the user to
click on one of them to change the view such that the clicked item is shown.

Your server must serve map tiles and handle search queries. Use open-source tools
for map tile generation and spatial databases for GIS data.  Download the data
for this assignment from here: https://grading.cse356.compas.cs.stonybrook.edu/data/new-york.osm.pbf

API Endpoints:

Map Tiles Endpoint: http://your.server/tiles/l$LAYER/$V/$H.png
Where $LAYER, $V, and $H represent the zoom level, vertical, and horizontal tile
indices, respectively.
Request Type: GET
Response: PNG image of the requested map tile.

Search Endpoint: http://your.server/api/search
Request Body (JSON):
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
Response (JSON): A list of objects within the bounding box matching the search term.
Each object includes the name and coordinates.  If onlyInBox is true, then only
objects within the query bbox are returned, with coordinates pointing to the
center of the VISIBLE PORTION of the object within the queried bounding box.
If onlyInBox is false, then coordinates are the center of the object and bbox is
the bounding box that includes the entire object.
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

Remember to add x-cse356 header.
HINT: Look up open-source databases capable of storing and querying GIS data.
HINT: For generating and serving map tiles, explore open-source tools mentioned
on the OpenStreetMap wiki.

## Solution

1. Run setup.sh
