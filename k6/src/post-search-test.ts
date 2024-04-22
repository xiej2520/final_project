import { check, fail } from 'k6';
import { Options } from 'k6/options';
import http from 'k6/http';

const DURATION = 30;
export let options: Options = {
  vus: 150,
  duration: `${DURATION}s`,
};

const rand = (l: number, h: number) => (Math.random() * (h - l) + l);
/// full us-northeast
//const minlat = 39;
//const maxlat = 47;
//const minlon = -80;
//const maxlon = -64;
/// new england
const minlat = 41.541478;
const maxlat = 44.898687;
const minlon = -74.742916;
const maxlon = -71.054832;
/// monaco
//const minlat = 43;
//const maxlat = 44;
//const minlon = 7;
//const maxlon = 8;

const locs: [string, number, number][] = [];
// new england is ~70% chance
// take your best guess on how many address + 2 * search queries can fit in the time
const ESTIMATED_ADDRESS_TIME = 0.5;
const ESTIMATED_SEARCH_TIME = 0.5;
const POPULATE_ATTEMPTS = DURATION / (ESTIMATED_ADDRESS_TIME + 2 * ESTIMATED_SEARCH_TIME) * 4;
console.log(`Attempting to retrieve ${POPULATE_ATTEMPTS} addresses`);
// disable auth to run
export default () => {
  /// build set on first run
  if (__ITER === 0) {
    for (let i=0; i<POPULATE_ATTEMPTS; i++) {
      // build object set
      const lat = rand(minlat, maxlat);
      const lon = rand(minlon, maxlon);
      const address_req = { lat, lon };
      const res = http.post('http://localhost/api/address', JSON.stringify(address_req), {
        headers: { 'Content-Type': 'application/json'},
      });
      if (res.status !== 200 || (typeof res.body !== 'string' && !(res.body instanceof String))) {
        continue;
      }
      const j = JSON.parse(res.body!.toString());
      if (j.name != undefined && j.name != "") {
        locs.push([String(j.name), lat, lon]);
      } else if (j.street != undefined && j.street != "") {
        locs.push([String(j.street), lat, lon]);
      } else if (j.city == undefined && j.state == undefined && j.country == undefined) {
        //console.log(address_req, j);
      }
    }
    console.log(`Found ${locs.length} locations`);
  }
  if (__ITER >= locs.length) {
    return;
    //fail("Ran out of locations to search for");
  }
  //console.log(`Found ${locs.length} locations for search testing`);
  {
    const anywhere_req = {
      bbox: {
        minLat: locs[__ITER][1],
        minLon: locs[__ITER][2],
        maxLat: locs[__ITER][1],
        maxLon: locs[__ITER][2],
      },
      onlyInBox: false,
      searchTerm: locs[__ITER][0]
    };
    const res = http.post('http://localhost/api/search', JSON.stringify(anywhere_req), {
      headers: { 'Content-Type': 'application/json'},
    });
    if (!check(res, { 'status is 200': r => r.status === 200, })) {
      fail('anywhere req status code was not 200');
    }
    if (!check(res, { 'valid response body': r => typeof r.body === 'string' || r.body instanceof String})) {
      fail("anywhere req invalid response body");
    }
    const j = JSON.parse(res.body!.toString());
    if (!Array.isArray(j)) {
      console.log(anywhere_req);
      fail(`${JSON.stringify(j)} is not an Array`);
    }
    const found = j.some((obj: any) => obj.name == locs[__ITER][0]);
    check(found, { 'Found anywhere search': f => f });
  }
  {
    const bbox_req = {
      bbox: {
        minLat: locs[__ITER][1] - 1,
        minLon: locs[__ITER][2] - 1,
        maxLat: locs[__ITER][1] + 1,
        maxLon: locs[__ITER][2] + 1,
      },
      onlyInBox: true,
      searchTerm: locs[__ITER][0]
    };
    const res = http.post('http://localhost/api/search', JSON.stringify(bbox_req), {
      headers: { 'Content-Type': 'application/json'},
    });
    if (!check(res, { 'status is 200': r => r.status === 200, })) {
      fail('status code was not 200');
    }
    if (!check(res, { 'valid response body': r => typeof r.body === 'string' || r.body instanceof String})) {
      fail("invalid response body");
    }
    const j = JSON.parse(res.body!.toString());
    const found = j.some((obj: any) => obj.name == locs[__ITER][0]);
    check(found, { 'Found bbox search': f => f });
    const allInBox = j.every((obj: any) =>
      locs[__ITER][1] - 1 <= obj.coordinates.lat && obj.coordinates.lat <= locs[__ITER][1] + 1 &&
      locs[__ITER][2] - 1 <= obj.coordinates.lon && obj.coordinates.lon <= locs[__ITER][2] + 1
    );
    check(allInBox, { 'All objects in bbox': f => f });
  }
};
