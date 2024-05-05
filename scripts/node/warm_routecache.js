const axios = require("axios");

const EPS = 0.25;

const X = [39.0, 47.0];
const Y = [-80.0, -67.0];
// 8 * 13 = 104
// (104 / eps^2)^2
// 416^2 = 173056 entries for EPS=0.5
// 1664^2 = 2768896 entries for EPS=0.25

async function foo(x, y) {
  const futures = [];
  for (let x1 = X[0]; x1 <= X[1] + EPS; x1 += EPS) {
    for (let y1 = Y[0]; y1 <= Y[1] + EPS; y1 += EPS) {
      const body = {
        source: { lat: x, lon: y },
        destination: { lat: x1, lon: y1 },
      };
      futures.push([axios.post("http://localhost/api/route", body), body]);
      if (futures.length > 100) {
        for (let f of futures) {
          try {
            console.log(
              f[1].source.lat,
              f[1].source.lon,
              f[1].destination.lat,
              f[1].destination.lon,
              (await f[0]).headers["x-cache-status"]
            );
          } catch (err) {
            console.log(err);
          }
        }
        futures = [];
      }
    }
  }
}

for (let x = X[0]; x <= X[1] + EPS; x += EPS) {
  for (let y = Y[0]; y <= Y[1] + EPS; y += EPS) {
    await foo(x, y);
    console.log(x, y);
  }
}
console.log("DONE");
