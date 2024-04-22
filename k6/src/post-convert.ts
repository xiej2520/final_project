import { check, fail, sleep } from 'k6';
import { Options } from 'k6/options';
import http from 'k6/http';

export let options: Options = {
  vus: 2500,
  duration: '10s',
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
const minzoom = 7;
const maxzoom = 22;

// disable auth to run
export default () => {
  const zoom = Math.floor(rand(minzoom, maxzoom));
  const convert_req = {
    lat: rand(minlat, maxlat),
    long: rand(minlon, maxlon),
    zoom: zoom,
  };
  const convert_response = http.post('http://localhost/convert', JSON.stringify(convert_req), {
    headers: { 'Content-Type': 'application/json'},
  });
  const convert_check = check(convert_response, {
    'convert status is 200': r => r.status === 200,
    'valid response body': r => {
      if (typeof r.body !== 'string' && !(r.body instanceof String) ) {
        return false;
      }
      const j = JSON.parse(r.body!.toString());
      if (typeof j.x_tile !== 'number' || typeof j.y_tile !== 'number') {
        return false;
      }
      return true;
    },
  });
  if (!convert_check) {
    fail("convert api failed");
  }
  //sleep(1);
};
