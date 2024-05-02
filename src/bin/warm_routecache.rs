#![feature(iter_array_chunks)]
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct RouteParams {
    source: Coordinates,
    destination: Coordinates,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Coordinates {
    lat: f64,
    lon: f64,
}

#[tokio::main]
async fn main() {
    let client = reqwest::Client::new();
    let X = [39.0, 47.0];
    let Y = [-80.0, -67.0];

    const EPS: f64 = 0.5;

    let mut x = X[0];
    async fn foo(client: Client, x: f64, y: f64) {
        let X = [39.0, 47.0];
        let Y = [-80.0, -67.0];
        let mut futures = vec![];
        let mut x1 = X[0];
        while x1 <= X[1] + EPS {
            let mut y1 = Y[0];
            while y1 <= Y[1] + EPS {
                let body = RouteParams {
                    source: Coordinates { lat: x, lon: y },
                    destination: Coordinates { lat: x1, lon: y1 },
                };
                let client = client.clone();
                futures.push(tokio::spawn( async move {
                    let resp = client
                        .post(format!(
                            "http://not-invented-here.cse356.compas.cs.stonybrook.edu/api/route"
                        ))
                        .json(&body)
                        .send().await;
                    println!("{:?}", resp.unwrap().bytes().await);
                    }
                ));
                if futures.len() > 100 {
                    for f in futures.into_iter() {
                        println!("{:?}", f.await.unwrap());
                    }
                }
                futures = vec![];
                y1 += EPS;
            }
            x1 += EPS;
            y1 = Y[0];
        }
    }
    while x <= X[1] + EPS {
        let mut y = Y[0];
        while y <= Y[1] + EPS {
            foo(client.clone(), x, y);
            y += EPS;
            println!("{x} {y}");
        }
        x += EPS;
        y = Y[0];
    }
    println!("DONE");
}
