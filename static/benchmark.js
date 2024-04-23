async function timeResponses(reqs, endpoint, parallel) {
  const remaining = document.getElementById("requestsRemaining");
  let numRemaining = reqs.length;
  remaining.innerHTML = numRemaining;
  const times = [];
  const start = performance.now();
  if (parallel) {
    await Promise.all(reqs.map(req => (async () => {
        const start = performance.now();
        await fetch(endpoint, {
          method: 'POST',
          headers: new Headers({ 'content-type': 'application/json' }),
          body: JSON.stringify(req)
        });
        times.push(performance.now() - start);
        numRemaining--;
        remaining.innerHTML = numRemaining;
      })()
    ));
  } else {
    for (const req of reqs) {
      const start = performance.now();
      await fetch(endpoint, {
        method: 'POST',
        headers: new Headers({ 'content-type': 'application/json' }),
        body: JSON.stringify(req)
      });
      times.push(performance.now() - start);
      numRemaining--;
      remaining.innerHTML = numRemaining;
    }
  }
  const total = performance.now() - start;
  times.sort((a, b) => a - b);
  return {
    total,
    times
  };
}

function drawPlot(id, times) {
  const n = times.length;
  const percentile = (percent) => times[Math.ceil((percent / 100) * n) - 1];
  const width = 2 * (percentile(75) - percentile(25)) * Math.pow(n, -1/3);

  const rounded = times.map(time => (Math.ceil(time / width) * width).toFixed(2));
  const freqMap = {};
  rounded.forEach(v => freqMap[v] = (freqMap[v] ?? 0) + 1);
  const freq = [];
  Object.entries(freqMap).forEach(([k, v]) => freq.push([parseFloat(k), parseInt(v)]));
  freq.sort((a, b) => a[0] - b[0]);
  const x = freq.map(([bin, freq]) => bin);
  const y = freq.map(([bin, freq]) => freq);

  const getColor = (value) => `hsl(${(value <= 5) ? 120 : (value >= 50 ? 0 : (120 - ((value - 5) / (50 - 5)) * 120))}, 100%, 50%)`;
  
  const ctx = document.getElementById(id).getContext('2d');
  let chartStatus = Chart.getChart("load-convert-chart");
  if (chartStatus != undefined) {
    chartStatus.destroy();
  }
  const plot = new Chart(ctx, {
    type: 'bar',
    data: {
        labels: x,
        datasets: [{
            label: 'times',
            data: y,
            //backgroundColor: 'rgba(20, 20, 40, 1)',
            backgroundColor: x.map(bin => getColor(bin)),
            //borderColor: x.map(bin => getColor2(bin)),
            //borderWidth: 1
        }]
    },
    options: {
        scales: {
            x: {
              title: {
                display: true,
                text: "Time (ms)",
              },
              beginAtZero: true
            },
            y: {
              beginAtZero: true
            }
        }
    }
  });
}
// times should be sorted
function showStats(id, benchmark) {
  const times = benchmark.times;
  const n = times.length;
  const min = times[0];
  const max = times[n - 1];
  const avg = times.reduce((acc, val) => acc + val, 0) / n;
  const median = n % 2 === 0 ? (times[n / 2 - 1] + times[n / 2]) / 2 : times[Math.floor(n / 2)];
  const percentile = (percent) => times[Math.ceil((percent / 100) * n) - 1];
  const p90 = percentile(90);
  const p95 = percentile(95);
  const p99 = percentile(99);
  document.getElementById(id).innerHTML = `
<tr> <th>Statistic</th> <th>Value</th> </tr>
<tr> <td>N</td> <td>${n}</td> </tr>
<tr> <td>Min</td> <td>${min.toFixed(2)}</td> </tr>
<tr> <td>Max</td> <td>${max.toFixed(2)}</td> </tr>
<tr> <td>Average</td> <td>${avg.toFixed(2)}</td> </tr>
<tr> <td>Median</td> <td>${median.toFixed(2)}</td> </tr>
<tr> <td>90th Percentile</td> <td>${p90.toFixed(2)}</td> </tr>
<tr> <td>95th Percentile</td> <td>${p95.toFixed(2)}</td> </tr>
<tr> <td>99th Percentile</td> <td>${p99.toFixed(2)}</td> </tr>
<tr> <td>Total</td> <td>${benchmark.total.toFixed(2)}</td> </tr>
`;
  }


// tests for convert and tiles
const convertTimes = [];
const tileTimes = [];
(() => {
const rand = (l, h) => (Math.random() * (h - l) + l);
function addBenchmark(benchmark, name) {
  drawPlot('load-convert-chart', benchmark.times);
  showStats('load-convert-table', benchmark);
  convertTimes.push(benchmark);
  const item = document.createElement("div");
  item.innerHTML = name;
  const i = list.children.length;
  item.addEventListener('click', (event) => {
    drawPlot('load-convert-chart', convertTimes[i].times);
    showStats('load-convert-table', convertTimes[i]);
  });
  document.getElementById("load-convert-samples").appendChild(item);
}
const form = document.getElementById("load-convert-form");
const list = document.getElementById("load-convert-samples");
form.onsubmit = async (event) => {
  event.preventDefault();
  const formData = new FormData(form);
  const data = Object.fromEntries(Array.from(formData).map(([k, v]) => [k, Number(v)]));
  console.log("Form data", data);

  if (event.submitter.value == "test-convert-seq") {
    const reqs = []
    for (let i=0; i<1000; i++) {
      reqs.push({
        lat: rand(data.minlat, data.maxlat),
        long: rand(data.minlon, data.maxlon),
        zoom: parseInt(rand(data.minzoom, data.maxzoom)),
      });
    }
    const benchmark = await timeResponses(reqs, "convert", false);
    addBenchmark(benchmark, "1000<br/>Convert<br/>Sequential");
  } else if (event.submitter.value == "test-convert-par") {
    const reqs = []
    for (let i=0; i<1000; i++) {
      reqs.push({
        lat: rand(data.minlat, data.maxlat),
        long: rand(data.minlon, data.maxlon),
        zoom: parseInt(rand(data.minzoom, data.maxzoom)),
      });
    }
    const benchmark = await timeResponses(reqs, "convert", true);
    addBenchmark(benchmark, "1000<br/>Convert<br/>Parallel");
  } else if (event.submitter.value == "test-tiles") {
    const reqs = []
    const start = performance.now();
    const times = [];
    const remaining = document.getElementById("requestsRemaining");
    const n = 1000;
    for (let i=0; i<n; i++) {
      const zoom = parseInt(rand(data.minzoom, data.maxzoom));
      const req = {
        lat: rand(data.minlat, data.maxlat),
        long: rand(data.minlon, data.maxlon),
        zoom
      };
      const resp = await fetch("convert", {
        method: 'POST',
        headers: new Headers({ 'content-type': 'application/json' }),
        body: JSON.stringify(req)
      });
      const json = await resp.json();
      const start = performance.now();
      const tileResp = await fetch(`/tiles/${zoom}/${json.x_tile}/${json.y_tile}.png`, {
        method: 'GET',
      });
      times.push(performance.now() - start);
      
      /*
      const imgBlob = await tileResp.blob();
      const imgObjectURL = URL.createObjectURL(imgBlob);
      const img = document.createElement('img');
      img.src = imgObjectURL;
      document.getElementById("content-convert").append(img);
      */
      remaining.innerHTML = n - i - 1;
    }
    const total = performance.now() - start;
    times.sort((a, b) => a - b);
    addBenchmark({total, times}, `${n}<br/>Tiles`);
  }
};
})();
