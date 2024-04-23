import { check, fail, sleep } from 'k6';
import { Options } from 'k6/options';
import http from 'k6/http';

//export let options: Options = {
//  vus: 500,
//  duration: '30s',
//};
export const options = {
  scenarios: {
    constant_request_rate: {
      executor: 'constant-arrival-rate',
      rate: 700,
      timeUnit: '1s',
      duration: '30s',
      preAllocatedVUs: 500,
      maxVUs: 1000,
    },
  },
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

// new england
const ranges = [
[], [], [], [], [], [], [], [], [], [], [], [], // 0..=11
[{x_tile:1195,y_tile:1474}, {x_tile:1241,y_tile:1539}],
[{x_tile:2390,y_tile:2948}, {x_tile:2483,y_tile:3079}],
[{x_tile:4780,y_tile:5897}, {x_tile:4966,y_tile:6159}],
[{x_tile:9561,y_tile:11795}, {x_tile:9932,y_tile:12318}],
[{x_tile:19123,y_tile:23590}, {x_tile:19865,y_tile:24636}],
[{x_tile:19123,y_tile:23590}, {x_tile:39731,y_tile:49272}],
[{x_tile:76493,y_tile:94360}, {x_tile:79462,y_tile:98544}]
];

// disable auth to run
export default () => {
  const zoom = Math.floor(rand(12, 18+1));
  const x_tile = Math.floor(rand(ranges[zoom][0].x_tile, ranges[zoom][1].x_tile));
  const y_tile = Math.floor(rand(ranges[zoom][0].y_tile, ranges[zoom][1].y_tile));

  const tile_res = http.get(`http://not-invented-here.cse356.compas.cs.stonybrook.edu/tiles/${zoom}/${x_tile}/${y_tile}.png`);
  check(tile_res, {
    'tile status is 200': r => r.status === 200,
    'Content-Type is png': r => r.headers['Content-Type'] === "image/png",
    'Got an interesting image': r => parseInt(r.headers['Content-Length']) >= 2000,
    'Got a very interesting image': r => parseInt(r.headers['Content-Length']) >= 10000,
  });
  //sleep(1);
};
