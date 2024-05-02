#![feature(iter_array_chunks)]
use reqwest::StatusCode;

#[tokio::main]
async fn main() {
    let ranges = [
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
        [[4528, 5742], [5100, 6216]],
        [[9057, 11485], [10201, 12432]],
        [[18115, 22971], [20402, 24864]],
    ];
    //[7, 16]

    for z in 7..=16 {
        println!("z: {z}");
        for x in ranges[z][0][0]..=ranges[z][1][0] {
            println!(" {z}: {x}");
            for chunk in (ranges[z][0][1]..=ranges[z][1][1]).array_chunks::<100>() {
                let futures: Vec<_> = chunk.iter()
                .map(|y| tokio::spawn(reqwest::get(format!("http://not-invented-here.cse356.compas.cs.stonybrook.edu/tiles/{z}/{x}/{y}.png"))))
                .collect();

                // do these futures in parallel and return them
                for f in futures.into_iter() {
                    f.await.unwrap();
                }
            }

            //for y in ranges[z][0][1]..=ranges[z][1][1] {
            //    let res = .await;
            //    if let Ok(res) = res {
            //        if res.status() != StatusCode::OK {
            //            println!("{z}/{x}/{y}: {}", res.status());
            //        }
            //    }
            //}
        }
    }
}
