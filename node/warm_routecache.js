const axios = require('axios');

const EPS = 0.25;

async function main() {
    const X = [39.0, 47.0];
    const Y = [-80.0, -67.0];

    async function foo(x, y) {
        const X = [39.0, 47.0];
        const Y = [-80.0, -67.0];
        let futures = [];
        let x1 = X[0];
        while (x1 <= X[1] + EPS) {
            let y1 = Y[0];
            while (y1 <= Y[1] + EPS) {
                const body = {
                    source: { lat: x, lon: y },
                    destination: { lat: x1, lon: y1 }
                };
                futures.push([axios.post('http://not-invented-here.cse356.compas.cs.stonybrook.edu/api/route', body), body]);
                //console.log(body);
                if (futures.length > 3) {
                    for (let f of futures) {
                        try {
                            await f[0];
                        } catch (err) {
                            console.log(f[1]);
                        }
                    }
                    futures = [];
                }
                y1 += EPS;
            }
            x1 += EPS;
            y1 = Y[0];
        }
    }

    let x = X[0];
    while (x <= X[1] + EPS) {
        let y = Y[0];
        while (y <= Y[1] + EPS) {
            await foo(x, y);
            y += EPS;
            console.log(`${x} ${y}`);
        }
        x += EPS;
        y = Y[0];
    }
    console.log("DONE");
}

main();
