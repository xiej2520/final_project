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
      rate: 1000,
      timeUnit: '1s',
      duration: '30s',
      preAllocatedVUs: 5000,
      maxVUs: 10000,
    },
  },
};

const rand = (l: number, h: number) => (Math.random() * (h - l) + l);
/// full us-northeast
const minlat = 39;
const maxlat = 47;
const minlon = -80;
const maxlon = -67;
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
const minzoom = 7;
const maxzoom = 18;

// full us-northeast
const ranges = [
[], [], [], [], [], [], [], // 0..=6
[{x_tile:34,y_tile:35},{x_tile:40,y_tile:49}],
[{x_tile:70,y_tile:89},{x_tile:79,y_tile:97}],
[{x_tile:141,y_tile:179},{x_tile:159,y_tile:194}],
[{x_tile:283,y_tile:358},{x_tile:318,y_tile:388}],
[{x_tile:566,y_tile:717},{x_tile:637,y_tile:777}],
[{x_tile:1132,y_tile:1435},{x_tile:1275,y_tile:1554}],
[{x_tile:2264,y_tile:2871},{x_tile:2550,y_tile:3108}],
[{x_tile:4528,y_tile:5742},{x_tile:5100,y_tile:6216}],
[{x_tile:9057,y_tile:11485},{x_tile:10201,y_tile:12432}],
[{x_tile:18115,y_tile:22971},{x_tile:20402,y_tile:24864}]
]

// new england
//const ranges = [
//[], [], [], [], [], [], [], [], [], [], [], [], // 0..=11
//[{x_tile:1195,y_tile:1474}, {x_tile:1241,y_tile:1539}],
//[{x_tile:2390,y_tile:2948}, {x_tile:2483,y_tile:3079}],
//[{x_tile:4780,y_tile:5897}, {x_tile:4966,y_tile:6159}],
//[{x_tile:9561,y_tile:11795}, {x_tile:9932,y_tile:12318}],
//[{x_tile:19123,y_tile:23590}, {x_tile:19865,y_tile:24636}],
//[{x_tile:38246,y_tile:47180}, {x_tile:39731,y_tile:49272}],
//[{x_tile:76493,y_tile:94360}, {x_tile:79462,y_tile:98544}]
//];

// disable auth to run
export default () => {
  const zoom = Math.floor(rand(7, 10));
  const x_tile = Math.floor(rand(ranges[zoom][0].x_tile, ranges[zoom][1].x_tile));
  const y_tile = Math.floor(rand(ranges[zoom][0].y_tile, ranges[zoom][1].y_tile));

  const tile_res = http.get(`http://not-invented-here.cse356.compas.cs.stonybrook.edu/tiles/${zoom}/${x_tile}/${y_tile}.png`);
  check(tile_res, {
    'tile status is 200': r => r.status === 200,
    'Content-Type is png': r => r.headers['Content-Type'] === "image/png",
    'Got an interesting image': r => parseInt(r.headers['Content-Length']) >= 2000,
    'Got a very interesting image': r => parseInt(r.headers['Content-Length']) >= 10000,
    'Got a cache hit': r => r.headers['X-Cache-Status'] === "HIT",
  });
  //sleep(1);
};
