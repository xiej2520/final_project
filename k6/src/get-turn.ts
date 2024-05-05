import { check } from 'k6';
import { Options } from 'k6/options';
import http from 'k6/http';

export let options: Options = {
  vus: 500,
  duration: '30s',
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
  const lat = [rand(minlat, maxlat), rand(minlat, maxlat)].sort();
  const lon = [rand(minlon, maxlon), rand(minlon, maxlon)].sort();
  const turn_res = http.get(`http://localhost/turn/${lat[0]},${lon[0]}/${lat[1]},${lon[0]}.png`);
  // check that image is 100x100?
  check(turn_res, {
    'turn status is 200': r => r.status === 200,
    'Content-Type is png': r => r.headers['Content-Type'] === "image/png",
    'Got an interesting image': r => parseInt(r.headers['Content-Length']) >= 2000,
    'Got a very interesting image': r => parseInt(r.headers['Content-Length']) >= 10000,
  });
};
