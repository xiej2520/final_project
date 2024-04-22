import { check, fail, sleep } from 'k6';
import { Options } from 'k6/options';
import http from 'k6/http';

export let options: Options = {
  vus: 1000,
  duration: '100s',
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

// disable auth to run
export default () => {
  const route_req ={
    source: {
      lat: rand(minlat, maxlat),
      lon: rand(minlon, maxlon),
    },
    destination: {
      lat: rand(minlat, maxlat),
      lon: rand(minlon, maxlon),
    },
  };
  const res = http.post('http://localhost/api/route', JSON.stringify(route_req), {
    headers: { 'Content-Type': 'application/json'},
  });
  if (!check(res, { 'status is 200': r => r.status === 200, })) {
    fail('status code was not 200');
  }
  check(res, {
    'valid response body': r => {
      if (typeof r.body !== 'string' && !(r.body instanceof String) ) {
        console.log("Body not a string");
        return false;
      }
      const j = JSON.parse(r.body!.toString());
      if (!Array.isArray(j)) {
        console.log("Parsed body is not array");
        return false;
      }
      for (const turn of j) {
        if (typeof turn.description !== 'string' && !(turn.description instanceof String) ) {
          console.log("Description not a string");
          return false;
        }
        if (turn.coordinates === undefined || turn.coordinates === null) {
          console.log("Coordinates not found");
          return false;
        }
        if (typeof turn.coordinates.lat !== 'number' || typeof turn.coordinates.lon !== 'number') {
          console.log("lat or lon not a number");
          return false;
        }
        if (typeof turn.distance !== 'number') {
          console.log("distance not a number");
          return false;
        }
      }
      return true;
    },
  });
  //sleep(1);
};
