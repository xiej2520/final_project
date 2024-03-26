# Milestone #1

## Description

Implement an interactive map viewer, including navigation (panning/scrolling)
and the ability to search for the named objects.  Search should be by object name
(e.g., address, street name, town name, etc.).  Search responses should be in
the order of distance from the center of the query bounding box.

The UI should show the results of the search.  If the search was requested with
onlyInBox true, then all results are within the current bounding box and they
should all be labeled with pins on top of the map without changing the view.  If
the search was requested with onlyInBox false, then if there is only one result,
the view should change to show that result in the center of the view, and if there
are multiple results, the UI should show the list of results and allow the user
to click on one of them to change the view such that the clicked item is shown.

Your server must serve map tiles and handle search queries. Use open-source tools
for map tile generation and spatial databases for GIS data.  Download the data
for this assignment from here: https://grading.cse356.compas.cs.stonybrook.edu/data/new-york.osm.pbf

API Endpoints:

Map Tiles Endpoint: http://your.server/tiles/$LAYER/$V/$H.png
Where $LAYER, $V, and $H represent the zoom level, vertical, and horizontal tile
indices, respectively. 
Note: We are forcing the tiles to be from left to right, up to down. If a tile x
is on the right of a tile y, then $V of x is greater than $V of y.
Request Type: GET
Response: PNG image of the requested map tile.

Search Endpoint: http://your.server/api/search
Request Type: POST
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
Response (JSON): A list of objects within the bounding box matching the search
term. Each object includes the name and coordinates.  If onlyInBox is true, then
only objects within the query bbox are returned, with coordinates pointing to the
center of the VISIBLE PORTION of the object within the queried bounding box.  If
onlyInBox is false, then coordinates are the center of the object and bbox is the
bounding box that includes the entire object.
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

Convert: http://your.server/convert
Request Type: POST
Request Body (JSON): given zoom level, latitude and longitude of a point, convert
to x y coordinate ($V $H) of tile indices that contains the point in Map Tiles Endpoint
e.g: if the zoom/lat/long of Manhattan is passed, return index of the tile that
contains Manhattan.
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

## Solution

1. Run setup.sh


* 4326 is lat/longitude
* 3857 is the data's srid
* .way is the geometry column
* `geometry ST_MakeEnvelope(float xmin, float ymin, float xmax, float ymax, integer srid=unknown)`

* transform lat/lon bbox to srid 3857 bbox,
* find polygons intersecting the bbox
* get name, centroid of intersection, transform Y and X back to lat and lon

```SQL
WITH transformed_bbox AS (
    SELECT ST_Transform(ST_MakeEnvelope(-73.135, 40.908, -73.100, 40.927, 4326), 3857) AS bbox
)
SELECT name,
       ST_Y(ST_Transform(ST_Centroid(intersection_geom), 4326)) AS lat,
       ST_X(ST_Transform(ST_Centroid(intersection_geom), 4326)) AS lon,
       ST_Distance(ST_Centroid(intersection_geom), ST_Centroid(transformed_bbox.bbox)) AS distance_to_center
FROM (
    SELECT planet_osm_polygon.name,
           ST_Intersection(planet_osm_polygon.way, transformed_bbox.bbox) AS intersection_geom
    FROM planet_osm_polygon, transformed_bbox
    WHERE 
      planet_osm_polygon.way && transformed_bbox.bbox
      AND name IS NOT NULL
) AS intersection_query, transformed_bbox
WHERE NOT ST_IsEmpty(intersection_geom)
-- AND NAME = 'Stony Brook University'
ORDER BY distance_to_center;
LIMIT 5000;
```

* Find any object matching name, return bounding box

```SQL
SELECT
    name,
    ST_Y(ST_Transform(centroid, 4326)) AS centroidLat,
    ST_X(ST_Transform(centroid, 4326)) AS centroidLon,
    ST_YMin(bbox) AS minLat,
    ST_XMin(bbox) AS minLon,
    ST_YMax(bbox) AS maxLat,
    ST_XMax(bbox) AS maxLon
FROM (
    SELECT 
        name,
        ST_Transform(ST_Envelope(ST_Collect(way)), 4326) AS bbox,
        ST_Centroid(ST_Collect(way)) AS centroid
    FROM (
        (SELECT name, way FROM planet_osm_polygon LIMIT 30)
        UNION
        (SELECT name, way FROM planet_osm_line LIMIT 30)
        UNION
        (SELECT name, way FROM planet_osm_roads LIMIT 30)
        UNION
        (SELECT name, way FROM planet_osm_point LIMIT 30)
    ) AS relevant_tables
    GROUP BY 
        name
) AS search_query
LIMIT 30;
```

                        name                         |        lat         |        lon         
-----------------------------------------------------+--------------------+--------------------
 Town of Brookhaven                                  |  40.91750068264421 |           -73.1175
 Long Island                                         |  40.91750068264421 |           -73.1175
 Suffolk County                                      |  40.91750068264421 |           -73.1175
 Hospital Parking Garage                             |  40.90809421302909 |   -73.116100844158
 Stony Brook University Hospital                     | 40.909961957970076 |  -73.1155600707645
 Health Sciences Center University Hospital Heliport | 40.909911923034926 | -73.11469103899496
 Hospital Tower                                      | 40.909205601439645 |  -73.1157066316318
 Health Sciences Tower                               |   40.9097093332288 | -73.11617201414171
 Basic Sciences Tower                                | 40.910317875733796 | -73.11676054187934
 Health Sciences Center                              |   40.9101822400949 |  -73.1165751739359
 Stony Brook Heights                                 |  40.91018825152585 |  -73.1172841008538
 Ashley Schiff Park Preserve                         |  40.90896121161942 | -73.12075081520035
 Stony Brook University                              |  40.91482544762445 | -73.12292746570193
 Stony Brook University                              |  40.91482544762445 | -73.12292746570193
 Forever Wild                                        |  40.91011569981644 | -73.12202842191874
 Roth/Tabler                                         |  40.91037295869728 | -73.12411827945084
 Freight Farm                                        |  40.91057789422243 | -73.12408100292211
 Roth/Tabler                                         |  40.91036539439625 |  -73.1245742601235
 Tabler Basketball                                   | 40.910258384820914 | -73.12540535246976
 Tabler Quad                                         |  40.91001316748208 | -73.12494772881693
 Express Route                                       |  40.91194635072084 | -73.12368458228947
 Clara's Woods                                       | 40.909029750892294 |  -73.1238268916368
 Stony Brook                                         | 40.919355018182884 | -73.13198315947517
 Dreiser College                                     |  40.90881182958601 | -73.12740209443332
 Tabler Quad                                         | 40.909471443359124 | -73.12703489027076
 Douglass College                                    |  40.90910151268864 | -73.12649181455957
 Hand College                                        | 40.909724332284405 | -73.12613665719373
 Tabler Tennis                                       |  40.91042478543245 |  -73.1257810492591
 Tabler Center for Arts, Culture, and Humanities     |  40.90987748147803 | -73.12707967642646
 Toscanini College                                   | 40.910218649083355 | -73.12792340147521
 Sanger College                                      |  40.90964191735815 | -73.12836496798984
 Roosevelt Quad                                      |  40.91203464390667 | -73.12996249340137
 Stimson College                                     |  40.91186912243195 | -73.12938606205368
 Roosevelt                                           |  40.91202994310985 | -73.12805324034989
 Tabler Steps                                        | 40.911055309274985 | -73.12599750307182
 Roosevelt                                           |  40.91191500481864 | -73.12762503581044
 Heavy Engineering                                   |  40.91251186543803 |  -73.1256906403295

          name          |    centroidlat    |    centroidlon     |      minlat       |       minlon       |       maxlat       |   maxlon    
------------------------+-------------------+--------------------+-------------------+--------------------+--------------------+-------------
 Stony Brook University | 40.90971783083373 | -73.12173372612413 | 40.89281730037999 | -73.13728619999999 | 40.925269900383704 | -73.1004066
