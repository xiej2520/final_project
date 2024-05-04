const axios = require('axios');

async function main() {
    const ranges = [
        [[0, 0], [0, 0]],
        [[0, 0], [0, 0]],
        [[0, 0], [0, 0]],
        [[0, 0], [0, 0]],
        [[0, 0], [0, 0]],
        [[0, 0], [0, 0]],
        [[0, 0], [0, 0]], // 0..=6
        [[34, 35], [40, 49]],
        [[70, 89], [79, 97]],
        [[141, 179], [159, 194]],
        [[283, 358], [318, 388]],
        [[566, 717], [637, 777]],
        [[1132, 1435], [1275, 1554]],
        [[2264, 2871], [2550, 3108]],
        [[4528, 5742], [5100, 6216]], // 1354824
        [[9057, 11485], [10201, 12432]], // 5416868
        [[18115, 22971], [20402, 24864]], // 21667472
    ];
    //[7, 16]

    for (let z = 7; z <= 15; z++) {
        console.log(`z: ${z}`);
        for (let x = ranges[z][0][0]; x <= ranges[z][1][0]; x++) {
            console.log(` ${z}: ${x}`);
            const diff = 2;
            for (let chunkStart = ranges[z][0][1]; chunkStart <= ranges[z][1][1]; chunkStart += diff) {
                console.log(chunkStart);
                const chunkEnd = Math.min(chunkStart + diff, ranges[z][1][1] + 1);
                const promises = [];
                for (let y = chunkStart; y < chunkEnd; y++) {
                    promises.push(axios.get(`http://not-invented-here.cse356.compas.cs.stonybrook.edu/tiles/${z}/${x}/${y}.png`));
                }
                await Promise.all(promises);
            }
        }
    }
}

main();
