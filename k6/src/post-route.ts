import { check, fail, sleep } from 'k6';
import http from 'k6/http';

export const options = {
  scenarios: {
    constant_request_rate: {
      executor: 'constant-arrival-rate',
      rate: 100,
      timeUnit: '1s',
      duration: '30s',
      preAllocatedVUs: 100,
      maxVUs: 1000,
    },
  },
};

const rand = (l: number, h: number) => (Math.random() * (h - l) + l);
/// full us-northeast
const minlat = 39;
const maxlat = 47;
const minlon = -80;
const maxlon = -67;
// 8 * 13 = 104
// (104 / eps)^2
// 208^2 = 43000
/// new england
//const minlat = 41.541478;
//const maxlat = 44.898687;
//const minlon = -74.742916;
//const maxlon = -71.054832;
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
  const res = http.post('http://not-invented-here.cse356.compas.cs.stonybrook.edu/api/route', JSON.stringify(route_req), {
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
    'Got a cache hit': r => r.headers['X-Cache-Status'] === "HIT",
  });
  //sleep(1);
};
