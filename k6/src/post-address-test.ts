import { check, fail } from 'k6';
import { Options } from 'k6/options';
import http from 'k6/http';

export let options: Options = {
  vus: 200,
  duration: '30s',
};

const rand = (l: number, h: number) => (Math.random() * (h - l) + l);
/// full us-northeast
//const minlat = 39;
//const maxlat = 47;
//const minlon = -80;
//const maxlon = -64;
/// new england
const minlat = 41.310824;
const maxlat = 44.980342;
const minlon = -74.742916;
const maxlon = -70.676541;
/// monaco
//const minlat = 43;
//const maxlat = 44;
//const minlon = 7;
//const maxlon = 8;

// disable auth to run
export default () => {
  const address_req ={
    lat: rand(minlat, maxlat),
    lon: rand(minlon, maxlon),
  };
  const res = http.post('http://localhost/api/address', JSON.stringify(address_req), {
    headers: { 'Content-Type': 'application/json'},
  });
  if (!check(res, { 'status is 200': r => r.status === 200, })) {
    fail('status code was not 200');
  }
  if (!check(res, { 'valid response body': r => typeof r.body === 'string' || r.body instanceof String})) {
    fail("invalid response body");
  }
  const j = JSON.parse(res.body!.toString());
  // === null since some responses don't have them
  check(j, { 'number': j => typeof j.number === 'string' || j.number instanceof String || j.number === null});
  check(j, { 'street': j => typeof j.street === 'string' || j.street instanceof String || j.street === null});
  check(j, { 'city': j => typeof j.city === 'string' || j.city instanceof String });
  check(j, { 'state': j => typeof j.state === 'string' || j.state instanceof String });
  check(j, { 'country': j => typeof j.country === 'string' || j.country instanceof String });
  //sleep(1);
};
